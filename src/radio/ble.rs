use embassy_futures::join::join;
use esp_hal::peripherals::BT;
use esp_radio::ble::controller::BleConnector;
use esp_radio::Controller as EspController;
use log::{info, warn};
use trouble_host::prelude::*;
use trouble_host::Controller;
use trouble_host::HostResources;

use crate::radio::service::ControlService;

const CONNECTIONS_MAX: usize = 1;
const L2CAP_CHANNELS_MAX: usize = 1;

#[gatt_server]
struct Server {
    control_service: ControlService,
}

type MyRunner<'d> = Runner<
    'd,
    ExternalController<BleConnector<'d>, 20>,
    StaticPacketPool<CriticalSectionRawMutex, _, _>,
>;

type MyPeripheral<'d> = Peripheral<
    'd,
    ExternalController<BleConnector<'d>, 20>,
    StaticPacketPool<CriticalSectionRawMutex, _, _>,
>;

pub async fn setup_ble(
    mut bluetooth_peripheral: BT<'static>,
    radio_controller: &'static EspController<'static>,
) -> (MyRunner<'_>, MyPeripheral<'_>) {
    let connector = BleConnector::new(radio_controller, bluetooth_peripheral.reborrow());
    let controller: ExternalController<_, 20> = ExternalController::new(connector);

    let mut resources: HostResources<DefaultPacketPool, CONNECTIONS_MAX, L2CAP_CHANNELS_MAX> =
        HostResources::new();
    let address: Address = Address::random([0xff, 0x8f, 0x1a, 0x05, 0xe4, 0xff]);
    let stack = trouble_host::new(controller, &mut resources).set_random_address(address);
    let Host {
        mut peripheral,
        runner,
        ..
    } = stack.build();

    return (runner, peripheral);
}

pub async fn start_advertise(runner: MyRunner, peripheral: MyPeripheral) {
    info!("Begin advertising!");

    let server = Server::new_with_config(GapConfig::Peripheral(PeripheralConfig {
        name: "TrouBLE",
        appearance: &appearance::REMOTE_CONTROL,
    }))
    .unwrap();

    let _ = join(ble_task(runner), async {
        loop {
            match advertise("Trouble Example", &mut peripheral, &server).await {
                Ok(conn) => {
                    let a = gatt_events_task(&server, &conn);
                }
                Err(e) => {
                    panic!("[adv] error: {:?}", e);
                }
            }
        }
    })
    .await;
}

async fn ble_task<C: Controller, P: PacketPool>(mut runner: Runner<'_, C, P>) {
    loop {
        if let Err(e) = runner.run().await {
            panic!("[ble_task] error: {:?}", e);
        }
    }
}

async fn advertise<'values, 'server, C: Controller>(
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

async fn gatt_events_task<P: PacketPool>(
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
