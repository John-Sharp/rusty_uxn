use crate::uxninterface::Uxn;
use crate::uxninterface::UxnError;

struct UxnWrapper<'a> {
    uxn: Box<&'a mut dyn Uxn>,
    push_fn: fn(&mut (dyn Uxn + 'a), u8) -> Result<(), UxnError>,
    push_ret_fn: fn(&mut (dyn Uxn + 'a), u8) -> Result<(), UxnError>,
    pop_fn: fn(&mut (dyn Uxn + 'a)) -> Result<u8, UxnError>,
    keep: bool,
    popped_values: Vec<u8>,
}

impl<'a> UxnWrapper<'a> {
    fn new(uxn: Box<&'a mut dyn Uxn>, keep: bool, ret: bool) -> Self {
        let push_fn = if ret == false {
            Uxn::push_to_working_stack
        } else {
            Uxn::push_to_return_stack
        };

        let push_ret_fn = if ret == false {
            Uxn::push_to_return_stack
        } else {
            Uxn::push_to_working_stack
        };

        let pop_fn = if ret == false {
            Uxn::pop_from_working_stack
        } else {
            Uxn::pop_from_return_stack
        };

        UxnWrapper {
            uxn,
            push_fn,
            push_ret_fn,
            pop_fn,
            keep,
            popped_values: Vec::new(),
        }
    }

    fn read_next_byte_from_ram(&mut self) -> Result<u8, UxnError> {
        self.uxn.read_next_byte_from_ram()
    }

    fn read_from_ram(&self, addr: u16) -> u8 {
        self.uxn.read_from_ram(addr)
    }

    fn push(&mut self, byte: u8) -> Result<(), UxnError> {
        // If in keep mode, popped_values will be populated with
        // what has been popped in the course of this operation.
        // Push these back on the stack to restore it to its
        // state prior to any pops before pushing the desired
        // value
        while let Some(val) = self.popped_values.pop() {
            (self.push_fn)(*self.uxn, val).expect("Couldn't push");
        }

        (self.push_fn)(*self.uxn, byte)
    }

    fn push_to_return_stack(&mut self, byte: u8) -> Result<(), UxnError> {
        (self.push_ret_fn)(*self.uxn, byte)
    }

    fn pop(&mut self) -> Result<u8, UxnError> {
        let popped = (self.pop_fn)(*self.uxn)?;

        if self.keep {
            self.popped_values.push(popped);
        }

        return Ok(popped);
    }

    fn pop_short(&mut self) -> Result<u16, UxnError> {
        let low = self.pop()?;
        let high = self.pop()?;

        Ok(u16::from_be_bytes([high, low]))
    }

    fn write_to_device(&mut self, device_address: u8, val: u8) {
        self.uxn.write_to_device(device_address, val)
    }
}

impl<'a> Drop for UxnWrapper<'a> {
    fn drop(&mut self) {
        // if in keep mode, popped values will be populated with
        // what has been popped in the course of this operation.
        // Push these back onto the stack to ensure they are kept
        while let Some(val) = self.popped_values.pop() {
            (self.push_fn)(*self.uxn, val).expect("Couldn't push");
        }
    }
}

mod stack;
pub use stack::lit_handler;
pub use stack::inc_handler;
pub use stack::pop_handler;
pub use stack::dup_handler;
pub use stack::nip_handler;
pub use stack::swp_handler;
pub use stack::ovr_handler;
pub use stack::rot_handler;

mod logic;
pub use logic::equ_handler;
pub use logic::neq_handler;
pub use logic::gth_handler;
pub use logic::lth_handler;

mod control_flow;
pub use control_flow::jmp_handler;
pub use control_flow::jcn_handler;
pub use control_flow::jsr_handler;
pub use control_flow::sth_handler;

mod memory;
pub use memory::ldz_handler;
pub use memory::stz_handler;
pub use memory::ldr_handler;
pub use memory::str_handler;
pub use memory::lda_handler;
pub use memory::sta_handler;
pub use memory::dei_handler;
pub use memory::deo_handler;

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;
    use std::collections::VecDeque;

    pub struct MockUxn {
        pub read_next_byte_from_ram_arguments_received: RefCell<VecDeque<()>>,
        pub read_next_byte_from_ram_values_to_return: RefCell<VecDeque<Result<u8, UxnError>>>,

        pub read_from_ram_arguments_received: RefCell<VecDeque<(u16,)>>,
        pub read_from_ram_values_to_return: RefCell<VecDeque<u8>>,

        pub get_program_counter_arguments_received: RefCell<VecDeque<()>>,
        pub get_program_counter_values_to_return: RefCell<VecDeque<Result<u16, UxnError>>>,

        pub set_program_counter_arguments_received: RefCell<VecDeque<(u16,)>>,

        pub push_to_return_stack_arguments_received: RefCell<VecDeque<(u8,)>>,
        pub push_to_return_stack_values_to_return: RefCell<VecDeque<Result<(), UxnError>>>,

        pub push_to_working_stack_arguments_received: RefCell<VecDeque<(u8,)>>,
        pub push_to_working_stack_values_to_return: RefCell<VecDeque<Result<(), UxnError>>>,

        pub pop_from_working_stack_arguments_received: RefCell<VecDeque<()>>,
        pub pop_from_working_stack_values_to_return: RefCell<VecDeque<Result<u8, UxnError>>>,

        pub pop_from_return_stack_arguments_received: RefCell<VecDeque<()>>,
        pub pop_from_return_stack_values_to_return: RefCell<VecDeque<Result<u8, UxnError>>>,

        pub write_to_device_arguments_received: RefCell<VecDeque<(u8, u8)>>,
    }

    impl MockUxn {
        pub fn new() -> Self {
            MockUxn {
                read_next_byte_from_ram_arguments_received: RefCell::new(VecDeque::new()),
                read_next_byte_from_ram_values_to_return: RefCell::new(VecDeque::from([
                    Ok(0xaa),
                    Ok(0xab),
                ])),

                read_from_ram_arguments_received: RefCell::new(VecDeque::new()),
                read_from_ram_values_to_return: RefCell::new(VecDeque::new()),

                get_program_counter_arguments_received: RefCell::new(VecDeque::new()),
                get_program_counter_values_to_return: RefCell::new(VecDeque::new()),

                set_program_counter_arguments_received: RefCell::new(VecDeque::new()),

                push_to_return_stack_arguments_received: RefCell::new(VecDeque::new()),
                push_to_return_stack_values_to_return: RefCell::new(VecDeque::new()),

                push_to_working_stack_arguments_received: RefCell::new(VecDeque::new()),
                push_to_working_stack_values_to_return: RefCell::new(VecDeque::new()),

                pop_from_working_stack_arguments_received: RefCell::new(VecDeque::new()),
                pop_from_working_stack_values_to_return: RefCell::new(VecDeque::new()),

                pop_from_return_stack_arguments_received: RefCell::new(VecDeque::new()),
                pop_from_return_stack_values_to_return: RefCell::new(VecDeque::new()),

                write_to_device_arguments_received: RefCell::new(VecDeque::new()),
            }
        }
    }

    impl Uxn for MockUxn {
        fn read_next_byte_from_ram(&mut self) -> Result<u8, UxnError> {
            self.read_next_byte_from_ram_arguments_received
                .borrow_mut()
                .push_back(());
            return self
                .read_next_byte_from_ram_values_to_return
                .borrow_mut()
                .pop_front()
                .unwrap();
        }

        fn read_from_ram(&self, addr: u16) -> u8 {
            self.read_from_ram_arguments_received
                .borrow_mut()
                .push_back((addr,));
            return self
                .read_from_ram_values_to_return
                .borrow_mut()
                .pop_front()
                .unwrap();
        }

        fn get_program_counter(&self) -> Result<u16, UxnError> {
            self.get_program_counter_arguments_received
                .borrow_mut()
                .push_back(());
            return self
                .get_program_counter_values_to_return
                .borrow_mut()
                .pop_front()
                .unwrap();
        }

        fn set_program_counter(&mut self, addr: u16) {
            self.set_program_counter_arguments_received
                .borrow_mut()
                .push_back((addr,));
        }

        fn push_to_return_stack(&mut self, byte: u8) -> Result<(), UxnError> {
            self.push_to_return_stack_arguments_received
                .borrow_mut()
                .push_back((byte,));

            return self
                .push_to_return_stack_values_to_return
                .borrow_mut()
                .pop_front()
                .unwrap();
        }

        fn push_to_working_stack(&mut self, byte: u8) -> Result<(), UxnError> {
            self.push_to_working_stack_arguments_received
                .borrow_mut()
                .push_back((byte,));

            return self
                .push_to_working_stack_values_to_return
                .borrow_mut()
                .pop_front()
                .unwrap();
        }

        fn pop_from_working_stack(&mut self) -> Result<u8, UxnError> {
            self.pop_from_working_stack_arguments_received
                .borrow_mut()
                .push_back(());
            return self
                .pop_from_working_stack_values_to_return
                .borrow_mut()
                .pop_front()
                .unwrap();
        }

        fn pop_from_return_stack(&mut self) -> Result<u8, UxnError> {
            self.pop_from_return_stack_arguments_received
                .borrow_mut()
                .push_back(());
            return self
                .pop_from_return_stack_values_to_return
                .borrow_mut()
                .pop_front()
                .unwrap();
        }

        fn write_to_device(&mut self, device_address: u8, val: u8) {
            self.write_to_device_arguments_received
                .borrow_mut()
                .push_back((device_address, val));
        }
    }

}
