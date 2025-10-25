pub enum PerformFunctionError {
    WrongFunctionCode,
    IncorrectAddress,
    InvalidValue,
}

// pub trait AddressParser<'a, T> {
//     fn parse_address() -> T;
// }

pub trait AddressablePeripheral<'a, T> {
    fn perform_function(
        self: &mut Self,
        function_code: u8,
        address: u8,
        value: u8,
    ) -> Result<T, PerformFunctionError>;
}
