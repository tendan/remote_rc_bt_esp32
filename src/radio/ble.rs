use core::future::Future;
use embassy_futures::select::{select, Either};

use embassy_time::Timer;
use log::{info, warn};
use trouble_host::prelude::*;

use crate::control::commands::{InstructionQueue, InstructionQueueSender};
use crate::control::instruction::{AddressablePeripheral, PerformFunctionError};
use crate::radio::service::ControlService;
use crate::radio::BLE_DEVICE_NAME;

#[gatt_server]
pub struct Server {
    pub control_service: ControlService,
}

pub(crate) fn create_ble_server<'v>() -> Server<'v> {
    Server::new_with_config(GapConfig::Peripheral(PeripheralConfig {
        name: "Remote RC BT",
        appearance: &appearance::power_device::GENERIC_POWER_DEVICE,
    }))
    .unwrap()
}

pub(crate) async fn ble_task<C: Controller, P: PacketPool>(mut runner: Runner<'_, C, P>) {
    loop {
        if let Err(e) = runner.run().await {
            panic!("[ble_task] error: {:?}", e);
        }
    }
}

pub enum BleState<'values, 'server, C: Controller> {
    Idle,
    Advertising,
    Connected(GattConnection<'values, 'server, DefaultPacketPool>),
    LostConnection,
    Err(BleHostError<C::Error>),
}

pub(crate) async fn advertise_while<'v, 's, A, C: Controller>(
    wait_for: A,
    peripheral: &mut Peripheral<'v, C, DefaultPacketPool>,
    server: &'s Server<'v>,
) -> BleState<'v, 's, C>
where
    A: Future,
{
    match select(
        begin_advertisement(BLE_DEVICE_NAME, peripheral, &server),
        wait_for,
    )
    .await
    {
        Either::First(Result::Ok(conn)) => BleState::Connected(conn),
        Either::First(Result::Err(e)) => BleState::Err(e),
        Either::Second(_) => BleState::Idle,
    }
}

pub(crate) async fn begin_advertisement<'values, 'server, C: Controller>(
    name: &'values str,
    peripheral: &mut Peripheral<'values, C, DefaultPacketPool>,
    server: &'server Server<'values>,
) -> Result<GattConnection<'values, 'server, DefaultPacketPool>, BleHostError<C::Error>> {
    let mut advertiser_data = [0; 31];
    let len = AdStructure::encode_slice(
        &[
            AdStructure::Flags(LE_GENERAL_DISCOVERABLE | BR_EDR_NOT_SUPPORTED),
            AdStructure::ServiceUuids16(&[[0x0f, 0x18]]),
            AdStructure::CompleteLocalName(name.as_bytes()),
        ],
        &mut advertiser_data[..],
    )?;
    let advertiser = peripheral
        .advertise(
            &Default::default(),
            Advertisement::ConnectableScannableUndirected {
                adv_data: &advertiser_data[..len],
                scan_data: &[],
            },
        )
        .await?;
    info!("[adv] advertising");
    let conn = advertiser.accept().await?.with_attribute_server(server)?;
    info!("[adv] connection established");
    Ok(conn)
}

pub(crate) async fn gatt_events_task<P: PacketPool>(
    server: &Server<'_>,
    conn: &GattConnection<'_, '_, P>,
) -> Result<(), Error> {
    let device_name = server.control_service.device_name;
    let reason = loop {
        match conn.next().await {
            GattConnectionEvent::Disconnected { reason } => break reason,
            GattConnectionEvent::Gatt { event } => {
                match &event {
                    GattEvent::Read(event) => {
                        if event.handle() == device_name.handle {
                            let value = server.get(&device_name);
                            info!("[gatt] Read Event to Level Characteristic: {:?}", value);
                        }
                    }
                    GattEvent::Write(event) => {
                        if event.handle() == device_name.handle {
                            info!(
                                "[gatt] Write Event to Level Characteristic: {:?}",
                                event.data()
                            );
                        }
                    }
                    _ => {}
                };
                // This step is also performed at drop(), but writing it explicitly is necessary
                // in order to ensure reply is sent.
                match event.accept() {
                    Ok(reply) => reply.send().await,
                    Err(e) => warn!("[gatt] error sending response: {:?}", e),
                };
            }
            _ => {} // ignore other Gatt Connection Events
        }
    };
    info!("[gatt] disconnected: {:?}", reason);
    Ok(())
}

// pub(crate) async fn custom_task<C: Controller, P: PacketPool>(
//     server: &Server<'_>,
//     conn: &GattConnection<'_, '_, P>,
//     stack: &Stack<'_, C, P>,
// ) {
//     let mut tick: u8 = 0;

//     let device_name = server.control_service.device_name;
//     loop {
//         tick = tick.wrapping_add(1);
//         info!("[custom_task] notifying connection of tick {}", tick);
//         if device_name.notify(conn, b"ESP32").await.is_err() {
//             info!("[custom_task] error notifying connection");
//             break;
//         };
//         // read RSSI (Received Signal Strength Indicator) of the connection.
//         if let Ok(rssi) = conn.raw().rssi(stack).await {
//             info!("[custom_task] RSSI: {:?}", rssi);
//         } else {
//             info!("[custom_task] error getting RSSI");
//             break;
//         };
//         Timer::after_secs(2).await;
//     }
// }

pub(crate) async fn steering_handle_task(
    server: &Server<'_>,
    instruction_queue: &InstructionQueueSender<'static>,
) {
    let steering = server.control_service.steering;
    match steering.set(server, &[0x0_u8, 0x0_u8, 0x0_u8, 0x0_u8]) {
        Ok(_) => info!("[steering_handle_task] Reset the steering register"),
        Err(_) => panic!("[steering_handle_task] Failed to reset steering"),
    }
    loop {
        // let Ok([/*peripheral_address*/_, function_code, port_address, value]) = steering.get(server) else { continue; };
        let Ok(instruction) = steering.get(server) else {
            continue;
        };
        instruction_queue.send(instruction).await;
        // info!(
        //     "[steering_handle_task] First byte: {}; Second byte: {}",
        //     first, second
        // );

        // Timer::after_millis(10).await;
    }
}
