#[macro_export]
macro_rules! load {
    ($account_loader: expr) => {{
        $account_loader.load().map_err(|_| {
            let error_code = ErrorCode::UnableToLoadAccountLoader;
            msg!("Error {} thrown at {}:{}", error_code, file!(), line!());
        })
    }};
}