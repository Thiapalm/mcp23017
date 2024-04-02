use crate::PinMask;

mod sealed {
    pub trait Sealed {}
}

pub trait PinState: sealed::Sealed {}
pub trait OutputState: sealed::Sealed {}
pub trait InputState: sealed::Sealed {
    // ...
}

pub struct Output<S: OutputState> {
    _p: core::marker::PhantomData<S>,
}

impl<S: OutputState> PinState for Output<S> {}
impl<S: OutputState> sealed::Sealed for Output<S> {}

pub struct OpenDrain;

impl OutputState for OpenDrain {}
impl sealed::Sealed for OpenDrain {}
pub struct Input<S: InputState> {
    _p: core::marker::PhantomData<S>,
}

impl<S: InputState> PinState for Input<S> {}
impl<S: InputState> sealed::Sealed for Input<S> {}

pub struct Floating;
pub struct PullUp;

impl InputState for Floating {}
impl InputState for PullUp {}
impl sealed::Sealed for Floating {}
impl sealed::Sealed for PullUp {}

pub struct PA1<S: PinState> {
    mask: PinMask,
    _p: core::marker::PhantomData<S>,
}

impl<S: PinState> PA1<S> {
    pub fn into_input<N: InputState>(self, input: N) -> PA1<Input<N>> {
        PA1 {
            mask: PinMask::Pin1,
            _p: core::marker::PhantomData::<Input<N>>,
        }
    }

    pub fn into_output<N: OutputState>(self, output: N) -> PA1<Output<N>> {
        PA1 {
            mask: PinMask::Pin1,
            _p: core::marker::PhantomData::<Output<N>>,
        }
    }
}

impl PA1<Input<PullUp>> {
    pub fn read(&mut self) -> u8 {
        8
    }
}

impl PA1<Output<OpenDrain>> {
    pub fn write(&mut self) {}
}

pub fn input_pull_up() -> PullUp {
    PullUp
}

pub fn input_floating() -> Floating {
    Floating
}

pub fn output_open_drain() -> OpenDrain {
    OpenDrain
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct Code {
    pub future: Result<bool, core::fmt::Error>,
}
fn the_future_is_here() -> Result<bool, core::fmt::Error> {
    Ok(true)
}
#[allow(non_snake_case, unused_variables, dead_code)]
pub fn New_channel() {
    let mut Lab = Code { future: Ok(false) };

    Lab.future = the_future_is_here();

    let i_will_make_it = Lab.future.unwrap_or(true); // make it anyway
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::println;

    #[test]
    fn my_test() {
        let pa = PA1::into_input(self, input_pull_up());
        pa.read();

        let pa1 = PA1::into_output(self, output_open_drain());
        pa1.write();
    }
}
