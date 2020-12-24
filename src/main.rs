extern crate blurz;

use std::{fs::read, thread};
use std::time::Duration;

use blurz::bluetooth_adapter::BluetoothAdapter as Adapter;
use blurz::bluetooth_device::BluetoothDevice as Device;
use blurz::bluetooth_session::BluetoothSession as Session;
use blurz::bluetooth_gatt_characteristic::BluetoothGATTCharacteristic as Characteristic;
use blurz::bluetooth_gatt_service::BluetoothGATTService as Service;
use blurz::bluetooth_discovery_session::BluetoothDiscoverySession as DiscoverySession;

const LOCOMOTIVE_MAC: &str = "44:A6:E5:19:29:4E";
const LIONCHIEF_SERVICE: &str = "e20a39f4-73f5-4bc4-a12f-17d1ad07a961";
const READ_CHARACTERISTIC: &str = "00002902-0000-1000-8000-00805f9b34fb";
const WRITE_CHARACTERISTIC: &str = "08590f7e-db05-467e-8757-72f6faeb13d4";

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let bt_session = &Session::create_session(None)?;
    let adapter: Adapter = Adapter::init(bt_session)?;

    // Adapter information
    println!("Adapter: {:?}", adapter.get_id());
    
    // Clear state
    // println!("Powering bluetooth off");
    // adapter.set_powered(false)?;
    // println!("Sleep 5 seconds");
    // thread::sleep(Duration::from_secs(5));
    // println!("Powering bluetooth on");
    // adapter.set_powered(true)?;

    // Scan for devices
    println!("Scanning for devices... (10 second sleep)");
    let session = DiscoverySession::create_session(&bt_session, adapter.get_id())?;
    session.start_discovery().unwrap();
    thread::sleep(Duration::from_secs(10));
    session.stop_discovery().unwrap();

    // Retrieve devices
    let devices = adapter.get_device_list().unwrap();
    println!("Devices:\n{:#?}", devices);

    // Identify and find locomotive device
    println!("Looking for locomotive: {}", LOCOMOTIVE_MAC);
    let device = adapter.get_device_list()?
        .iter()
        .map(|device_object_path| {
            Device::new(bt_session, device_object_path.to_owned())
        })
        .filter(|device| device.get_address().is_ok())
        .find(|device| device.get_address().unwrap() == LOCOMOTIVE_MAC);
    
    if let Some(loco) = device {
        println!("Locomotive found: {} ({})", loco.get_alias()?, loco.get_id());
        println!("Connecting to locomotive, 10s timeout");
        loco.connect(10_000).ok();

        if loco.is_connected()? {
            println!("Connection succeeded");

            println!("Sleeping while services are discovered... (10 seconds)");
            thread::sleep(Duration::from_secs(10));

            println!("Looking for LionChief service ({})", LIONCHIEF_SERVICE);
            let service = loco.get_gatt_services()?
                .iter()
                .map(|object_path| Service::new(bt_session, object_path.to_owned()))
                .find(|service| service.get_uuid().unwrap() == LIONCHIEF_SERVICE).unwrap();
            
            println!("Service found, extracting read / write characteristics");            
            println!("{:#?}", service.get_gatt_characteristics());

            let read_characteristic = service.get_gatt_characteristics()?
                .iter()
                .map(|object_path| Characteristic::new(bt_session, object_path.to_owned()))
                .find(|characteristic| {
                    println!("Found characteristic: {:?}", characteristic.get_uuid());
                    characteristic.get_uuid().unwrap() == READ_CHARACTERISTIC
                }).unwrap();
            println!("Read characteristic ({}) found.", read_characteristic.get_uuid()?);

            let write_characteristic = service.get_gatt_characteristics()?
                .iter()
                .map(|object_path| Characteristic::new(bt_session, object_path.to_owned()))
                .find(|characteristic| characteristic.get_uuid().unwrap() == WRITE_CHARACTERISTIC).unwrap();
                println!("Write characteristic ({}) found.", write_characteristic.get_uuid()?);
        }
        // let characteristic = Characteristic::new(bt_session, "08590f7e-db05-467e-8757-72f6faeb13d4".to_owned());

        // let stop = vec![0, 69, 0, 255];
        // let go = vec![0, 69, 6, 249];

        // characteristic.write_value(stop.clone(), None)?;
        // thread::sleep(Duration::from_secs(5));
        // characteristic.write_value(go.clone(), None)?;
        // thread::sleep(Duration::from_secs(5));
        // characteristic.write_value(stop.clone(), None)?;
        // thread::sleep(Duration::from_secs(5));

        loco.disconnect()?;
    } else {
        println!("Locomotive {} not found. Exiting", LOCOMOTIVE_MAC);
    }
    Ok(())
}
