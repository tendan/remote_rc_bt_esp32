use embassy_futures::join::join;
use embassy_futures::select::{select, select3, Either};
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::signal::Signal;
use embassy_time::{Duration, Timer};
use log::info;
use trouble_host::prelude::*;

use crate::hardware::board::Board;
use crate::hardware::motor::Motors;
use crate::radio::ble::*;
use config::*;

mod ble;
mod config;
mod service;

//
// Do przodu, do tyłu, lewo, prawo
// TODO:
// Diody, aplikacja, opis pracy (pousuwać pesymizm), czy chętni na demo (do twórców TrouBLE), opis README

pub async fn start_ble<C>(
    controller: C,
    ble_advertisement_signal: &'static Signal<CriticalSectionRawMutex, bool>,
    mut motors: Motors<'static>,
) where
    C: Controller,
{
    let address: Address = Address::random([0xff, 0x8f, 0x1a, 0x05, 0xe4, 0xff]);
    info!("Our address = {:?}", address);

    let mut resources: HostResources<DefaultPacketPool, CONNECTIONS_MAX, L2CAP_CHANNELS_MAX> =
        HostResources::new();

    let stack = trouble_host::new(controller, &mut resources).set_random_address(address);
    let Host {
        mut peripheral,
        runner,
        ..
    } = stack.build();

    info!("Starting advertising and GATT service");
    let server = create_ble_server();

    let _ = join(ble_task(runner), async {
        let mut state: BleState<'_, '_, C> = BleState::Idle;

        loop {
            match state {
                BleState::Idle => {
                    info!("[start_ble] Idle state");
                    let advertisement_enabled = ble_advertisement_signal.wait().await;
                    if advertisement_enabled {
                        state = BleState::Advertising;
                    };
                    Timer::after_secs(1).await;
                }
                BleState::Advertising => {
                    info!("[start_ble] Advertising state");
                    state =
                        advertise_while(ble_advertisement_signal.wait(), &mut peripheral, &server)
                            .await;

                    if matches!(state, BleState::Idle) {
                        info!("[start_ble] Cancelling advertisement");
                    }
                }
                BleState::Connected(conn) => {
                    info!("[start_ble] Connected state");
                    // set up tasks when the connection is established to a central, so they don't run when no one is connected.
                    let a = gatt_events_task(&server, &conn);
                    // let b = custom_task(&server, &conn, &stack);
                    let c = steering_handle_task(&server, &mut motors);
                    // run until any task ends (usually because the connection has been closed),
                    // then return to advertising state.

                    select(a, c).await;
                    state = BleState::LostConnection;
                }
                BleState::LostConnection => {
                    info!("[start_ble] Lost connection state");
                    motors.stop();
                    state = advertise_while(
                        Timer::after(Duration::from_secs(20)),
                        &mut peripheral,
                        &server,
                    )
                    .await;

                    if matches!(state, BleState::Idle) {
                        info!("[start_ble] Could not reconnect");
                    }
                }
                BleState::Err(e) => {
                    panic!("[start_ble] error: {:?}", e);
                }
            }
        }
    })
    .await;
}
