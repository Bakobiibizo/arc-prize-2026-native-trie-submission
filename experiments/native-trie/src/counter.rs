//! Seeded counter fixture. Layout vectors exist only for loading/inspection, never for cycle().
use crate::{Address, Machine, Outcome, install};

pub struct Bit {
    pub cell: Address,
    pub zero: Address,
    pub one: Address,
}
pub struct Counter {
    pub machine: Machine,
    pub entry: Address,
    pub returned: Address,
    pub overflowed: Address,
    pub bits: Vec<Bit>,
}

pub fn prepare(width: usize, ones: bool) -> Counter {
    assert!(width > 0);
    let machine = Machine::default();
    let root = machine.states.root();
    let returned = root.provision(b"returned");
    returned.terminate();
    let overflowed = root.provision(b"overflowed");
    overflowed.terminate();
    let return_link = root.provision(b"return-link");
    return_link.point_to(&returned);
    let overflow_link = root.provision(b"overflow-link");
    overflow_link.point_to(&overflowed);
    let states = root.provision(b"bits");
    let output = machine.output.root();
    let mut bits = Vec::with_capacity(width);
    for number in 0..width {
        let token = number.to_be_bytes();
        let location = states.provision(&token);
        let zero = location.provision(b"zero");
        let one = location.provision(b"one");
        let cell = output.provision(&token);
        cell.point_to(if ones { &one } else { &zero });
        bits.push(Bit { cell, zero, one });
    }
    for (number, bit) in bits.iter().enumerate() {
        let to_zero = bit.zero.provision(b"zero-value");
        to_zero.point_to(&bit.zero);
        let to_one = bit.one.provision(b"one-value");
        to_one.point_to(&bit.one);
        install(&bit.zero, &to_one, &bit.cell, &return_link);
        install(
            &bit.one,
            &to_zero,
            &bit.cell,
            bits.get(number + 1)
                .map(|b| &b.cell)
                .unwrap_or(&overflow_link),
        );
    }
    let entry = root.provision(b"pulse");
    let scratch = root.provision(b"scratch");
    install(&entry, &bits[0].cell, &scratch, &bits[0].cell);
    Counter {
        machine,
        entry,
        returned,
        overflowed,
        bits,
    }
}

impl Counter {
    pub fn pulse(&mut self, limit: usize) -> Outcome {
        self.machine.enter(&self.entry);
        for _ in 0..limit {
            let outcome = self.machine.cycle().expect("invalid seeded counter");
            if outcome == Outcome::Halted {
                return outcome;
            }
        }
        panic!("counter cycle limit exhausted: {limit}")
    }
    pub fn inspect(&self) -> Vec<bool> {
        self.bits
            .iter()
            .map(|bit| {
                let state = bit.cell.follow().expect("unbound count bit");
                assert!(state.same(&bit.zero) || state.same(&bit.one));
                state.same(&bit.one)
            })
            .collect()
    }
}
