use hidapi::HidApi;

pub struct Context {
    _hid_api: HidApi
}

impl Context {
    pub fn new() -> Result<Context,String> {
        let _hid_api = match HidApi::new() {
            Err(e) => {
                eprintln!("Error: {}", e);
                return Err(format!("{}",e));
            },
            Ok(api) => api,
        };
        Ok(Context{ _hid_api })
    }
}

pub fn available_nscopes(_ctx: &Context) {
    println!("Printing all available nScope devices:");

    for d in _ctx._hid_api.device_list() {
        if d.product_id() == 0xf3f6 && d.vendor_id() == 0x04d8 {
            println!("0x{:04x}, 0x{:04x}", d.vendor_id(), d.product_id())

        }
    }
}
