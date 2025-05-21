defmt::timestamp!("{=u32:us}", 0xDEADBEEF);

fn main() {
    defmt::println!(
        "hello: {} - {}",
        5,
        Test {
            a: 1,
            b: "2",
            c: 3.0
        }
    );
}

#[derive(defmt::Format)]
struct Test {
    a: u32,
    b: &'static str,
    c: f64,
}

#[defmt::global_logger]
struct Logger;

unsafe impl defmt::Logger for Logger {
    fn acquire() {}
    unsafe fn flush() {}
    unsafe fn release() {}
    unsafe fn write(_bytes: &[u8]) {
        println!("-- write: {_bytes:02X?}");
    }
}

