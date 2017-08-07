/*
Copyright (c) 2016-2017, Robert Ou <rqou@robertou.com> and contributors
All rights reserved.

Redistribution and use in source and binary forms, with or without
modification, are permitted provided that the following conditions are met:

1. Redistributions of source code must retain the above copyright notice,
   this list of conditions and the following disclaimer.
2. Redistributions in binary form must reproduce the above copyright notice,
   this list of conditions and the following disclaimer in the documentation
   and/or other materials provided with the distribution.

THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS "AS IS" AND
ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED TO, THE IMPLIED
WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR PURPOSE ARE
DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT HOLDER OR CONTRIBUTORS BE LIABLE
FOR ANY DIRECT, INDIRECT, INCIDENTAL, SPECIAL, EXEMPLARY, OR CONSEQUENTIAL
DAMAGES (INCLUDING, BUT NOT LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR
SERVICES; LOSS OF USE, DATA, OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER
CAUSED AND ON ANY THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY,
OR TORT (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE
OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.
*/

//! Tool that converts a jed to a json file

use std::cell::RefCell;
use std::collections::HashMap;
use std::fs::File;
use std::io::Read;

extern crate xc2bit;
use xc2bit::*;

extern crate yosys_netlist_json;
use yosys_netlist_json::*;

fn main() {
    let args = ::std::env::args().collect::<Vec<_>>();

    if args.len() != 2 {
        println!("Usage: {} file.jed", args[0]);
        ::std::process::exit(1);
    }

    // Load the jed
    let mut f = File::open(&args[1]).expect("failed to open file");
    let mut data = Vec::new();
    f.read_to_end(&mut data).expect("failed to read data");

    let bits_result = read_jed(&data);
    let (bits, device_name_option) = bits_result.expect("failed to read jed");
    let device_name = device_name_option.expect("missing device name in jed");

    let bitstream_result = XC2Bitstream::from_jed(&bits, &device_name);
    let bitstream = bitstream_result.expect("failed to process jed");

    // Because of how the closures work, we unfortunately need to use a RefCell here
    let output_netlist = RefCell::new(Netlist::default());
    let node_vec = RefCell::new(Vec::new());
    // Skip 0 and 1 just like yosys does
    let wire_idx = RefCell::new(2);

    // Create common/global stuff ahead of time
    {
        let mut output_netlist_mut = output_netlist.borrow_mut();
        // TOOD: Proper version number
        output_netlist_mut.creator = String::from("xc2bit xc2jed2json utility - DEVELOPMENT VERSION");

        let mut module = Module::default();
        module.attributes.insert(String::from("PART_NAME"),
            AttributeVal::S(format!("{}", bitstream.bits.device_type())));
        module.attributes.insert(String::from("PART_SPEED"),
            AttributeVal::S(format!("{}", bitstream.speed_grade)));
        module.attributes.insert(String::from("PART_PKG"),
            AttributeVal::S(format!("{}", bitstream.package)));

        output_netlist_mut.modules.insert(String::from("top"), module);
    }

    // Call the giant structure callback
    get_device_structure(bitstream.bits.device_type(),
        |node_name: &str, node_type: &str, fb: u32, idx: u32| {
            // Start constructing the cell object
            let cell;

            match node_type {
                "BUFG" | "BUFGSR" | "BUFGTS" => {
                    let mut connections = HashMap::new();
                    connections.insert(String::from("I"), Vec::new());
                    connections.insert(String::from("O"), Vec::new());
                    cell = Cell {
                        hide_name: 0,
                        cell_type: node_type.to_owned(),
                        parameters: HashMap::new(),
                        attributes: HashMap::new(),
                        port_directions: HashMap::new(),
                        connections: connections,
                    }
                },
                "IOBUFE" => {
                    // Construct the wire for the toplevel port
                    let toplevel_wire = {
                        let orig_wire_idx = {*wire_idx.borrow()};
                        let mut output_netlist_mut = output_netlist.borrow_mut();
                        let mut netnames = &mut output_netlist_mut.modules.get_mut("top").unwrap().netnames;
                        netnames.insert(format!("PAD_{}", idx), Netname {
                            hide_name: 0,
                            bits: vec![BitVal::N(orig_wire_idx)],
                            attributes: HashMap::new(),
                        });

                        // Return the current wire index
                        *wire_idx.borrow_mut() += 1;
                        orig_wire_idx
                    };

                    // Construct the toplevel port
                    {
                        let mut output_netlist_mut = output_netlist.borrow_mut();
                        let mut ports = &mut output_netlist_mut.modules.get_mut("top").unwrap().ports;
                        ports.insert(format!("PAD_{}", idx), Port {
                            direction: PortDirection::InOut,
                            bits: vec![BitVal::N(toplevel_wire)],
                        });
                    }

                    let mut connections = HashMap::new();
                    connections.insert(String::from("I"), Vec::new());
                    connections.insert(String::from("E"), Vec::new());
                    connections.insert(String::from("O"), Vec::new());
                    connections.insert(String::from("IO"), vec![BitVal::N(toplevel_wire)]);
                    cell = Cell {
                        hide_name: 0,
                        cell_type: node_type.to_owned(),
                        parameters: HashMap::new(),
                        attributes: HashMap::new(),
                        port_directions: HashMap::new(),
                        connections: connections,
                    }
                },
                // FIXME: Boilerplate
                "IBUF" => {
                    // Construct the wire for the toplevel port
                    let toplevel_wire = {
                        let orig_wire_idx = {*wire_idx.borrow()};
                        let mut output_netlist_mut = output_netlist.borrow_mut();
                        let mut netnames = &mut output_netlist_mut.modules.get_mut("top").unwrap().netnames;
                        netnames.insert(String::from("INPAD"), Netname {
                            hide_name: 0,
                            bits: vec![BitVal::N(orig_wire_idx)],
                            attributes: HashMap::new(),
                        });

                        // Return the current wire index
                        *wire_idx.borrow_mut() += 1;
                        orig_wire_idx
                    };

                    // Construct the toplevel port
                    {
                        let mut output_netlist_mut = output_netlist.borrow_mut();
                        let mut ports = &mut output_netlist_mut.modules.get_mut("top").unwrap().ports;
                        ports.insert(String::from("INPAD"), Port {
                            direction: PortDirection::Input,
                            bits: vec![BitVal::N(toplevel_wire)],
                        });
                    }

                    let mut connections = HashMap::new();
                    connections.insert(String::from("O"), Vec::new());
                    connections.insert(String::from("I"), vec![BitVal::N(toplevel_wire)]);
                    cell = Cell {
                        hide_name: 0,
                        cell_type: node_type.to_owned(),
                        parameters: HashMap::new(),
                        attributes: HashMap::new(),
                        port_directions: HashMap::new(),
                        connections: connections,
                    }
                },
                _ => {
                    // FIXME
                    println!("{} is not implemented!", node_type);
                    cell = Cell {
                        hide_name: 0,
                        cell_type: String::from("FIXME FIXME FIXME"),
                        parameters: HashMap::new(),
                        attributes: HashMap::new(),
                        port_directions: HashMap::new(),
                        connections: HashMap::new(),
                    }
                },
            }

            // Create the cell in the output module
            let mut output_netlist_mut = output_netlist.borrow_mut();
            let mut cells = &mut output_netlist_mut.modules.get_mut("top").unwrap().cells;
            cells.insert(node_name.to_owned(), cell);

            // Memoization needed for the callback interface
            let mut node_vec = node_vec.borrow_mut();
            node_vec.push((node_name.to_owned(), node_type.to_owned(), fb, idx));
            node_vec.len() - 1
        },
        |wire_name: &str| {
            // Create the net in the output module
            let orig_wire_idx = {*wire_idx.borrow()};
            let mut output_netlist_mut = output_netlist.borrow_mut();
            let mut netnames = &mut output_netlist_mut.modules.get_mut("top").unwrap().netnames;
            netnames.insert(wire_name.to_owned(), Netname {
                hide_name: 0,
                bits: vec![BitVal::N(orig_wire_idx)],
                attributes: HashMap::new(),
            });

            // Return the current wire index
            *wire_idx.borrow_mut() += 1;
            orig_wire_idx
        },
        |node_ref: usize, wire_ref: usize, port_name: &str, port_idx: u32| {
        }
    );

    // Write the final output
    output_netlist.borrow().to_writer(&mut ::std::io::stdout()).expect("failed to write json");
}
