#![macro_use]

#[macro_export]
macro_rules! ifelse {
    // with 3 arguments, act like a ternary
    ($condition: expr, $true: expr, $false: expr) => {
        if $condition { $true } else { $false }
    };

    // with 2 arguments, return the second one as an option
    ($condition: expr, $some: expr) => {
        if $condition { Some($some) } else { None }
    };

    // with 1 arg, default to bool
    ($condition: expr) => {
        if $condition { true } else { false }
    };
}

// shortcut for the most common print command I use
// if n = 3, log!(n) logs n: 3
#[macro_export]
macro_rules! log {
    ($i: ident) => {
        println!("{}: {:#?}", stringify!($i), $i)
    };
}