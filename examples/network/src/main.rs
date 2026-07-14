#![no_main]
#![no_std]

use ariel_os::log::info;

use ariel_os::{log::*, net};

#[ariel_os::task(autostart)]
async fn main() {

    let stack = net::network_stack().await.unwrap();
    info!("waiting for interface to come up...");
    
    stack.wait_config_up().await;

    // {
    //     let config = stack.config_v4().expect("Must work at all cost");
    //     let addres = config.address.address();
    //     info!("addres V4 = {}", addres);
    // }

    {
        let config = stack.config_v6().expect("Must work at all cost");
        let addres = config.address.address();
        info!("addres V6 = {}", addres); 
    }

    loop {}
}