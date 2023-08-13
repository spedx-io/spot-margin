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

#[macro_export]
macro_rules! safe_increement {
    ($value:expr, $struct:expr) => {{
        $struct = $struct.checked_add($value).ok_or_else(ErrorCode::SafeIncrementError)?;
    }};
}

#[macro_export]
macro_rules! safe_decrement {
    ($value:expr, $struct:expr) => {{
        $struct = $struct.checked_sub($value).ok_or_else(ErrorCode::SafeDecrementError)?;
    }};
}

#[macro_export]
macro_rules! update_struct_id {
    ($struct:expr, $property:ident) => {{
        let curr_id = $struct.$property;

        $struct.$property = curr_id.checked_add(1).or(Some(1).unwrap());
        curr_id
    }};
}