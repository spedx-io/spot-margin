// use crate::error::ErrorCode;

#[macro_export]
macro_rules! load {
    ($account_loader: expr) => {{
        $account_loader.load().map_err(|_| {
            let error_code = ErrorCode::UnableToLoadAccountLoader;
            msg!("Error {} thrown at {}:{}", error_code, file!(), line!());
        })
    }};
}

#[macro_export]
macro_rules! validate {
    ($assert:expr, $err:expr) => {{
        if ($assert) {
            Ok(())
        } else {
            let error_code: ErrorCode = $err;
            msg!("Error {} thrown at {}:{}", error_code, file!(), line!());
            Err(error_code)
        }
    }};
    ($assert:expr, $err:expr, $($arg:tt)+) => {{
        if($assert) {
            Ok(())
        } else{
            let error_code: ErrorCode = $err;
            msg!("Error {} thrown at {}:{}", error_code, file!(), line!());
            msg!($($arg)*);
            Err(error_code) 
        }
    }};
}