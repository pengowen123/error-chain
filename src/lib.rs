#![deny(missing_docs)]
#![allow(unknown_lints)] // to be removed when unused_doc_comments lints is merged
#![doc(html_root_url = "https://docs.rs/error-chain/0.11.0")]

//! A library for consistent and reliable error handling
//!
//! error-chain makes it easy to take full advantage of Rust's
//! powerful error handling features without the overhead of
//! maintaining boilerplate error types and conversions. It implements
//! an opinionated strategy for defining your own error types, as well
//! as conversions from others' error types.
//!
//! ## Quick start
//!
//! If you just want to set up your new project with error-chain,
//! follow the [quickstart.rs] template, and read this [intro]
//! to error-chain.
//!
//! [quickstart.rs]: https://github.com/rust-lang-nursery/error-chain/blob/master/examples/quickstart.rs
//! [intro]: http://brson.github.io/2016/11/30/starting-with-error-chain
//!
//! ## Why error chain?
//!
//! * error-chain is easy to configure. Handle errors robustly with minimal
//!   effort.
//! * Basic error handling requires no maintenance of custom error types
//!   nor the [`From`] conversions that make `?` work.
//! * error-chain scales from simple error handling strategies to more
//!   rigorous.  Return formatted strings for simple errors, only
//!   introducing error variants and their strong typing as needed for
//!   advanced error recovery.
//! * error-chain makes it trivial to correctly manage the [cause] of
//!   the errors generated by your own code. This is the "chaining"
//!   in "error-chain".
//!
//! [cause]: https://doc.rust-lang.org/std/error/trait.Error.html#method.cause
//!
//! ## Principles of error-chain
//!
//! error-chain is based on the following principles:
//!
//! * No error should ever be discarded. This library primarily
//!   makes it easy to "chain" errors with the [`chain_err`] method.
//! * Introducing new errors is trivial. Simple errors can be introduced
//!   at the error site with just a string.
//! * Handling errors is possible with pattern matching.
//! * Conversions between error types are done in an automatic and
//!   consistent way - [`From`] conversion behavior is never specified
//!   explicitly.
//! * Errors implement [`Send`].
//! * Errors can carry backtraces.
//!
//! Similar to other libraries like [error-type] and [quick-error],
//! this library introduces the error chaining mechanism originally
//! employed by Cargo.  The [`error_chain!`] macro declares the types
//! and implementation boilerplate necessary for fulfilling a
//! particular error-handling strategy. Most importantly it defines a
//! custom error type (called [`Error`] by convention) and the [`From`]
//! conversions that let the `?` operator work.
//!
//! This library differs in a few ways from previous error libs:
//!
//! * Instead of defining the custom [`Error`] type as an enum, it is a
//!   struct containing an [`ErrorKind`][] (which defines the
//!   [`description`] and [`display_chain`] methods for the error), an opaque,
//!   optional, boxed [`std::error::Error`]` + `[`Send`]` + 'static` object
//!   (which defines the [`cause`], and establishes the links in the
//!   error chain), and a [`Backtrace`].
//! * The macro also defines a [`ResultExt`] trait that defines a
//!   [`chain_err`] method. This method on all [`std::error::Error`]` + `[`Send`]` + 'static`
//!   types extends the error chain by boxing the current
//!   error into an opaque object and putting it inside a new concrete
//!   error.
//! * It provides automatic [`From`] conversions between other error types
//!   defined by the [`error_chain!`] that preserve type information,
//!   and facilitate seamless error composition and matching of composed
//!   errors.
//! * It provides automatic [`From`] conversions between any other error
//!   type that hides the type of the other error in the [`cause`] box.
//! * If `RUST_BACKTRACE` is enabled, it collects a single backtrace at
//!   the earliest opportunity and propagates it down the stack through
//!   [`From`] and [`ResultExt`] conversions.
//!
//! To accomplish its goals it makes some tradeoffs:
//!
//! * The split between the [`Error`] and [`ErrorKind`] types can make it
//!   slightly more cumbersome to instantiate new (unchained) errors,
//!   requiring an [`Into`] or [`From`] conversion; as well as slightly
//!   more cumbersome to match on errors with another layer of types
//!   to match.
//! * Because the error type contains [`std::error::Error`]` + `[`Send`]` + 'static` objects,
//!   it can't implement [`PartialEq`] for easy comparisons.
//!
//! ## Declaring error types
//!
//! Generally, you define one family of error types per crate, though
//! it's also perfectly fine to define error types on a finer-grained
//! basis, such as per module.
//!
//! Assuming you are using crate-level error types, typically you will
//! define an `errors` module and inside it call [`error_chain!`]:
//!
//! ```
//! # #[macro_use] extern crate error_chain;
//! mod other_error {
//!     error_chain! {}
//! }
//!
//! error_chain! {
//!     // The type defined for this error. These are the conventional
//!     // and recommended names, but they can be arbitrarily chosen.
//!     //
//!     // It is also possible to leave this section out entirely, or
//!     // leave it empty, and these names will be used automatically.
//!     types {
//!         Error, ErrorKind, ResultExt, Result;
//!     }
//!
//!     // Without the `Result` wrapper:
//!     //
//!     // types {
//!     //     Error, ErrorKind, ResultExt;
//!     // }
//!
//!     // Automatic conversions between this error chain and other
//!     // error chains. In this case, it will e.g. generate an
//!     // `ErrorKind` variant called `Another` which in turn contains
//!     // the `other_error::ErrorKind`, with conversions from
//!     // `other_error::Error`.
//!     //
//!     // Optionally, some attributes can be added to a variant.
//!     //
//!     // This section can be empty.
//!     links {
//!         Another(other_error::Error, other_error::ErrorKind) #[cfg(unix)];
//!     }
//!
//!     // Automatic conversions between this error chain and other
//!     // error types not defined by the `error_chain!`. These will be
//!     // wrapped in a new error with, in the first case, the
//!     // `ErrorKind::Fmt` variant. The description and cause will
//!     // forward to the description and cause of the original error.
//!     //
//!     // Optionally, some attributes can be added to a variant.
//!     //
//!     // This section can be empty.
//!     foreign_links {
//!         Fmt(::std::fmt::Error);
//!         Io(::std::io::Error) #[cfg(unix)];
//!     }
//!
//!     // Define additional `ErrorKind` variants.  Define custom responses with the
//!     // `description` and `display` calls.
//!     errors {
//!         InvalidToolchainName(t: String) {
//!             description("invalid toolchain name")
//!             display("invalid toolchain name: '{}'", t)
//!         }
//!
//!         // You can also add commas after description/display.
//!         // This may work better with some editor auto-indentation modes:
//!         UnknownToolchainVersion(v: String) {
//!             description("unknown toolchain version"), // note the ,
//!             display("unknown toolchain version: '{}'", v), // trailing comma is allowed
//!         }
//!     }
//! }
//!
//! # fn main() {}
//! ```
//!
//! Each section, `types`, `links`, `foreign_links`, and `errors` may
//! be omitted if it is empty.
//!
//! This populates the module with a number of definitions,
//! the most important of which are the [`Error`] type
//! and the [`ErrorKind`] type. An example of generated code can be found in the
//! [example_generated](example_generated/index.html) module.
//!
//! ## Returning new errors
//!
//! Introducing new error chains, with a string message:
//!
//! ```
//! # #[macro_use] extern crate error_chain;
//! # fn main() {}
//! # error_chain! {}
//! fn foo() -> Result<()> {
//!     Err("foo error!".into())
//! }
//! ```
//!
//! Introducing new error chains, with an [`ErrorKind`]:
//!
//! ```
//! # #[macro_use] extern crate error_chain;
//! # fn main() {}
//! error_chain! {
//!     errors { FooError }
//! }
//!
//! fn foo() -> Result<()> {
//!     Err(ErrorKind::FooError.into())
//! }
//! ```
//!
//! Note that the return type is the typedef [`Result`], which is
//! defined by the macro as `pub type Result<T> =
//! ::std::result::Result<T, Error>`. Note that in both cases
//! [`.into()`] is called to convert a type into the [`Error`] type; both
//! strings and [`ErrorKind`] have [`From`] conversions to turn them into
//! [`Error`].
//!
//! When the error is emitted behind the `?` operator, the explicit conversion
//! isn't needed; `Err(ErrorKind)` will automatically be converted to `Err(Error)`.
//! So the below is equivalent to the previous:
//!
//! ```
//! # #[macro_use] extern crate error_chain;
//! # fn main() {}
//! # error_chain! { errors { FooError } }
//! fn foo() -> Result<()> {
//!     Ok(Err(ErrorKind::FooError)?)
//! }
//!
//! fn bar() -> Result<()> {
//!     Ok(Err("bogus!")?)
//! }
//! ```
//!
//! ## The `bail!` macro
//!
//! The above method of introducing new errors works but is a little
//! verbose. Instead, we can use the [`bail!`] macro, which performs an early return
//! with conversions done automatically.
//!
//! With [`bail!`] the previous examples look like:
//!
//! ```
//! # #[macro_use] extern crate error_chain;
//! # fn main() {}
//! # error_chain! { errors { FooError } }
//! fn foo() -> Result<()> {
//!     if true {
//!         bail!(ErrorKind::FooError);
//!     } else {
//!         Ok(())
//!     }
//! }
//!
//! fn bar() -> Result<()> {
//!     if true {
//!         bail!("bogus!");
//!     } else {
//!         Ok(())
//!     }
//! }
//! ```
//!
//! ## Chaining errors
//! error-chain supports extending an error chain by appending new errors.
//! This can be done on a Result or on an existing Error.
//!
//! To extend the error chain:
//!
//! ```
//! # #[macro_use] extern crate error_chain;
//! # fn main() {}
//! # error_chain! {}
//! # fn do_something() -> Result<()> { unimplemented!() }
//! # fn test() -> Result<()> {
//! let res: Result<()> = do_something().chain_err(|| "something went wrong");
//! # Ok(())
//! # }
//! ```
//!
//! [`chain_err`] can be called on any [`Result`] type where the contained
//! error type implements [`std::error::Error`]` + `[`Send`]` + 'static`, as long as
//! the [`Result`] type's corresponding [`ResultExt`] trait is in scope.  If
//! the [`Result`] is an `Err` then [`chain_err`] evaluates the closure,
//! which returns *some type that can be converted to [`ErrorKind`]*,
//! boxes the original error to store as the cause, then returns a new
//! error containing the original error.
//!
//! Calling [`chain_err`][Error_chain_err] on an existing [`Error`] instance has
//! the same signature and produces the same outcome as being called on a
//! [`Result`] matching the properties described above. This is most useful when
//! partially handling errors using the [`map_err`] function.
//!
//! To chain an error directly, use [`with_chain`]:
//!
//! ```
//! # #[macro_use] extern crate error_chain;
//! # fn main() {}
//! # error_chain! {}
//! # fn do_something() -> Result<()> { unimplemented!() }
//! # fn test() -> Result<()> {
//! let res: Result<()> =
//!     do_something().map_err(|e| Error::with_chain(e, "something went wrong"));
//! # Ok(())
//! # }
//! ```
//!
//! ## Linking errors
//!
//! To convert an error from another error chain to this error chain:
//!
//! ```
//! # #[macro_use] extern crate error_chain;
//! # fn main() {}
//! # mod other { error_chain! {} }
//! error_chain! {
//!     links {
//!         OtherError(other::Error, other::ErrorKind);
//!     }
//! }
//!
//! fn do_other_thing() -> other::Result<()> { unimplemented!() }
//!
//! # fn test() -> Result<()> {
//! let res: Result<()> = do_other_thing().map_err(|e| e.into());
//! # Ok(())
//! # }
//! ```
//!
//! The [`Error`] and [`ErrorKind`] types implements [`From`] for the corresponding
//! types of all linked error chains. Linked errors do not introduce a new
//! cause to the error chain.
//!
//! ## Matching errors
//!
//! error-chain error variants are matched with simple patterns.
//! [`Error`] is a tuple struct and its first field is the [`ErrorKind`],
//! making dispatching on error kinds relatively compact:
//!
//! ```
//! # #[macro_use] extern crate error_chain;
//! # fn main() {
//! error_chain! {
//!     errors {
//!         InvalidToolchainName(t: String) {
//!             description("invalid toolchain name")
//!             display("invalid toolchain name: '{}'", t)
//!         }
//!     }
//! }
//!
//! match Error::from("error!") {
//!     Error(ErrorKind::InvalidToolchainName(_), _) => { }
//!     Error(ErrorKind::Msg(_), _) => { }
//!     _ => { }
//! }
//! # }
//! ```
//!
//! Chained errors are also matched with (relatively) compact syntax
//!
//! ```
//! # #[macro_use] extern crate error_chain;
//! mod utils {
//!     error_chain! {
//!         errors {
//!                 description("bad stuff")
//!             }
//!         }
//!     }
//! }
//!
//! mod app {
//!     error_chain! {
//!         links {
//!             Utils(::utils::Error, ::utils::ErrorKind);
//!         }
//!     }
//! }
//!
//!
//! # fn main() {
//! match app::Error::from("error!") {
//!     app::Error(app::ErrorKind::Utils(utils::ErrorKind::BadStuff), _) => { }
//!     _ => { }
//! }
//! # }
//! ```
//!
//! ## Inspecting errors
//!
//! An error-chain error contains information about the error itself, a backtrace, and the chain
//! of causing errors. For reporting purposes, this information can be accessed as follows.
//!
//! ```
//! # #[macro_use] extern crate error_chain;
//! use error_chain::ChainedError;  // for e.display_chain()
//!
//! error_chain! {
//!     errors {
//!         InvalidToolchainName(t: String) {
//!             description("invalid toolchain name")
//!             display("invalid toolchain name: '{}'", t)
//!         }
//!     }
//! }
//!
//! # fn main() {
//! // Generate an example error to inspect:
//! let e = "xyzzy".parse::<i32>()
//!     .chain_err(|| ErrorKind::InvalidToolchainName("xyzzy".to_string()))
//!     .unwrap_err();
//!
//! // Get the brief description of the error:
//! assert_eq!(e.description(), "invalid toolchain name");
//!
//! // Get the display version of the error:
//! assert_eq!(e.to_string(), "invalid toolchain name: 'xyzzy'");
//!
//! // Get the full cause and backtrace:
//! println!("{}", e.display_chain().to_string());
//! //     Error: invalid toolchain name: 'xyzzy'
//! //     Caused by: invalid digit found in string
//! //     stack backtrace:
//! //        0:     0x7fa9f684fc94 - backtrace::backtrace::libunwind::trace
//! //                             at src/backtrace/libunwind.rs:53
//! //                              - backtrace::backtrace::trace<closure>
//! //                             at src/backtrace/mod.rs:42
//! //        1:     0x7fa9f6850b0e - backtrace::capture::{{impl}}::new
//! //                             at out/capture.rs:79
//! //     [..]
//! # }
//! ```
//!
//! The [`Error`] and [`ErrorKind`] types also allow programmatic access to these elements.
//!
//! ## Foreign links
//!
//! Errors that do not conform to the same conventions as this library
//! can still be included in the error chain. They are considered "foreign
//! errors", and are declared using the `foreign_links` block of the
//! [`error_chain!`] macro. [`Error`]s are automatically created from
//! foreign errors by the `?` operator.
//!
//! Foreign links and regular links have one crucial difference:
//! [`From`] conversions for regular links *do not introduce a new error
//! into the error chain*, while conversions for foreign links *always
//! introduce a new error into the error chain*. So for the example
//! above all errors deriving from the [`std::fmt::Error`] type will be
//! presented to the user as a new [`ErrorKind`] variant, and the
//! cause will be the original [`std::fmt::Error`] error. In contrast, when
//! `other_error::Error` is converted to `Error` the two `ErrorKind`s
//! are converted between each other to create a new `Error` but the
//! old error is discarded; there is no "cause" created from the
//! original error.
//!
//! ## Backtraces
//!
//! If the `RUST_BACKTRACE` environment variable is set to anything
//! but ``0``, the earliest non-foreign error to be generated creates
//! a single backtrace, which is passed through all [`From`] conversions
//! and [`chain_err`] invocations of compatible types. To read the
//! backtrace just call the [`backtrace`] method.
//!
//! Backtrace generation can be disabled by turning off the `backtrace` feature.
//!
//! The Backtrace contains a Vec of [`BacktraceFrame`]s that can be operated
//! on directly.  For example, to only see the files and line numbers of code
//! within your own project.
//!
//! ```
//! # #[macro_use]
//! # extern crate error_chain;
//! # mod errors {
//! #   error_chain! {
//! #       foreign_links {
//! #           Io(::std::io::Error);
//! #       }
//! #   }
//! # }
//! # use errors::*;
//! # #[cfg(feature="backtrace")]
//! # fn main() {
//! if let Err(ref e) = open_file() {
//!     if let Some(backtrace) = e.backtrace() {
//!         let frames = backtrace.frames();
//!         for frame in frames.iter() {
//!             for symbol in frame.symbols().iter() {
//!                 if let (Some(file), Some(lineno)) = (symbol.filename(), symbol.lineno()) {
//!                     if file.display().to_string()[0..3] == "src".to_string(){
//!                         println!("{}:{}", file.display().to_string(), lineno);
//!                     }
//!                 }
//!             }
//!         }
//!     }
//! };
//! # }
//! # #[cfg(not(feature="backtrace"))]
//! # fn main() { }
//!
//! fn open_file() -> Result<()> {
//!    std::fs::File::open("does_not_exist")?;
//!    Ok(())
//! }
//! ```
//!
//! ## Iteration
//!
//! The [`iter`] method returns an iterator over the chain of error boxes.
//!
//! [error-type]: https://github.com/DanielKeep/rust-error-type
//! [quick-error]: https://github.com/tailhook/quick-error

//! [`display_chain`]: trait.ChainedError.html#method.display_chain
//! [`error_chain!`]: macro.error_chain.html
//! [`bail!`]: macro.bail.html
//! [`Backtrace`]: struct.Backtrace.html

//! [`Error`]: example_generated/struct.Error.html
//! [`with_chain`]: example_generated/struct.Error.html#method.with_chain
//! [Error_chain_err]: example_generated/struct.Error.html#method.chain_err
//! [`cause`]: example_generated/struct.Error.html#method.cause
//! [`backtrace`]: example_generated/struct.Error.html#method.backtrace
//! [`iter`]: example_generated/struct.Error.html#method.iter
//! [`ErrorKind`]: example_generated/enum.ErrorKind.html
//! [`description`]: example_generated/enum.ErrorKind.html#method.description
//! [`Result`]: example_generated/type.Result.html
//! [`ResultExt`]: example_generated/trait.ResultExt.html
//! [`chain_err`]: example_generated/trait.ResultExt.html#tymethod.chain_err

//! [`std::error::Error`]: https://doc.rust-lang.org/std/error/trait.Error.html
//! [`Send`]: https://doc.rust-lang.org/std/marker/trait.Send.html
//! [`Into`]: https://doc.rust-lang.org/std/convert/trait.Into.html
//! [`From`]: https://doc.rust-lang.org/std/convert/trait.From.html
//! [`PartialEq`]: https://doc.rust-lang.org/std/cmp/trait.PartialEq.html
//! [`std::fmt::Error`]: https://doc.rust-lang.org/std/fmt/struct.Error.html
//! [`.into()`]: https://doc.rust-lang.org/std/convert/trait.Into.html#tymethod.into
//! [`map_err`]: https://doc.rust-lang.org/std/result/enum.Result.html#method.map_err
//! [`BacktraceFrame`]: https://docs.rs/backtrace/0.3.2/backtrace/struct.BacktraceFrame.html

use std::error;
use std::iter::Iterator;
use std::fmt;
use std::marker::PhantomData;

#[macro_use]
mod impl_error_chain_kind;
#[macro_use]
mod error_chain;
#[macro_use]
mod quick_main;
pub use quick_main::ExitCode;
#[cfg(feature = "example_generated")]
pub mod example_generated;
mod backtrace;
pub use backtrace::Backtrace;
#[doc(hidden)]
pub use backtrace::InternalBacktrace;

#[derive(Debug)]
/// Iterator over the error chain using the `Error::cause()` method.
pub struct Iter<'a>(Option<&'a error::Error>);

impl<'a> Iter<'a> {
    /// Returns a new iterator over the error chain using `Error::cause()`.
    pub fn new(err: Option<&'a error::Error>) -> Iter<'a> {
        Iter(err)
    }
}

impl<'a> Iterator for Iter<'a> {
    type Item = &'a error::Error;

    fn next<'b>(&'b mut self) -> Option<&'a error::Error> {
        match self.0.take() {
            Some(e) => {
                self.0 = e.cause();
                Some(e)
            }
            None => None,
        }
    }
}

/// This trait is implemented on all the errors generated by the `error_chain`
/// macro.
pub trait ChainedError<S: ?Sized>: error::Error + Send + 'static {
    /// Associated kind type.
    type ErrorKind;

    /// Constructs an error from a kind, and generates a backtrace.
    fn from_kind(kind: Self::ErrorKind) -> Self
    where
        Self: Sized;

    /// Constructs a chained error from another error and a kind, and generates a backtrace.
    fn with_chain<E, K>(error: E, kind: K) -> Self
    where Self: Sized,
          E: ToError + ::std::error::Error + Send + 'static,
          K: Into<Self::ErrorKind>;

    /// Returns the kind of the error.
    fn kind(&self) -> &Self::ErrorKind;

    /// Iterates over the error chain.
    fn iter(&self) -> Iter;

    /// Returns the backtrace associated with this error.
    fn backtrace(&self) -> Option<&Backtrace>;

    /// Returns an object which implements `Display` for printing the full
    /// context of this error.
    ///
    /// The full cause chain and backtrace, if present, will be printed.
    fn display_chain<'a>(&'a self) -> DisplayChain<'a, Self> {
        DisplayChain(self, PhantomData)
    }

    /// Extends the error chain with a new entry.
    fn chain_err<F, EK>(self, error: F) -> Self
    where
        F: FnOnce() -> EK,
        EK: Into<Self::ErrorKind>;

    /// Creates an error from its parts.
    #[doc(hidden)]
    fn new(kind: Self::ErrorKind, state: State<S>) -> Self
    where
        Self: Sized;

    /// Returns the first known backtrace, either from its State or from one
    /// of the errors from `foreign_links`.
    #[doc(hidden)]
    fn extract_backtrace(e: &(error::Error + Send + 'static)) -> Option<InternalBacktrace>;
}

pub trait ToError {
    fn to_error(&self) -> &(error::Error + Send + 'static);
}

/// A struct which formats an error for output.
#[derive(Debug)]
pub struct DisplayChain<'a, T: 'a + ?Sized, S: ?Sized>(&'a T, PhantomData<S>);

impl<'a, T, S> fmt::Display for DisplayChain<'a, T, S>
where
    T: ChainedError<S>,
{
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        // Keep `try!` for 1.10 support
        try!(writeln!(fmt, "Error: {}", self.0));

        for e in self.0.iter().skip(1) {
            try!(writeln!(fmt, "Caused by: {}", e));
        }

        if let Some(backtrace) = self.0.backtrace() {
            try!(writeln!(fmt, "{:?}", backtrace));
        }

        Ok(())
    }
}

/// Common state between errors.
#[derive(Debug)]
#[doc(hidden)]
pub struct State<T: ?Sized> {
    /// Next error in the error chain.
    pub next_error: Option<Box<T>>,
    /// Backtrace for the current error.
    pub backtrace: InternalBacktrace,
}

impl<T: ?Sized> Default for State<T> {
    #[cfg(feature = "backtrace")]
    fn default() -> Self {
        State {
            next_error: None,
            backtrace: InternalBacktrace::new(),
        }
    }
}

impl<T> State<T>
where
    T: error::Error + Send + 'static,
{
    /// Creates a new State type
    pub fn new<CE: ChainedError>(e: Box<error::Error + Send>) -> State {
        let backtrace = CE::extract_backtrace(&*e).unwrap_or_else(InternalBacktrace::new);
        State {
            next_error: Some(e),
            backtrace: backtrace,
        }
    }
}

/// Exits a function early with an error
///
/// The `bail!` macro provides an easy way to exit a function.
/// `bail!(expr)` is equivalent to writing.
///
/// ```
/// # #[macro_use] extern crate error_chain;
/// # error_chain! { }
/// # fn main() { }
/// # fn foo() -> Result<()> {
/// # let expr = "";
///     return Err(expr.into());
/// # }
/// ```
///
/// And as shorthand it takes a formatting string a la `println!`:
///
/// ```
/// # #[macro_use] extern crate error_chain;
/// # error_chain! { }
/// # fn main() { }
/// # fn foo() -> Result<()> {
/// # let n = 0;
/// bail!("bad number: {}", n);
/// # }
/// ```
///
/// # Examples
///
/// Bailing on a custom error:
///
/// ```
/// # #[macro_use] extern crate error_chain;
/// # fn main() {}
/// error_chain! {
///     errors { FooError }
/// }
///
/// fn foo() -> Result<()> {
///     if bad_condition() {
///         bail!(ErrorKind::FooError);
///     }
///
///     Ok(())
/// }
///
/// # fn bad_condition() -> bool { true }
/// ```
///
/// Bailing on a formatted string:
///
/// ```
/// # #[macro_use] extern crate error_chain;
/// # fn main() {}
/// error_chain! { }
///
/// fn foo() -> Result<()> {
///     if let Some(bad_num) = bad_condition() {
///         bail!("so bad: {}", bad_num);
///     }
///
///     Ok(())
/// }
///
/// # fn bad_condition() -> Option<i8> { None }
/// ```
#[macro_export]
macro_rules! bail {
    ($e:expr) => {
        return Err($e.into());
    };
    ($fmt:expr, $($arg:tt)+) => {
        return Err(format!($fmt, $($arg)+).into());
    };
}

/// Exits a function early with an error if the condition is not satisfied
///
/// The `ensure!` macro is a convenience helper that provides a way to exit
/// a function with an error if the given condition fails.
///
/// As an example, `ensure!(condition, "error code: {}", errcode)` is equivalent to
///
/// ```
/// # #[macro_use] extern crate error_chain;
/// # error_chain! { }
/// # fn main() { }
/// # fn foo() -> Result<()> {
/// # let errcode = 0u8;
/// # let condition = true;
/// if !condition {
///     bail!("error code: {}", errcode);
/// }
/// # Ok(())
/// # }
/// ```
///
/// See documentation for `bail!` macro for further details.
#[macro_export]
macro_rules! ensure {
    ($cond:expr, $e:expr) => {
        if !($cond) {
            bail!($e);
        }
    };
    ($cond:expr, $fmt:expr, $($arg:tt)+) => {
        if !($cond) {
            bail!($fmt, $($arg)+);
        }
    };
}

#[doc(hidden)]
pub mod mock {
    error_chain!{}
}
