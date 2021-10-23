pub fn main() {
    let (client, _) = jack::Client::new("lv2jack", jack::ClientOptions::NO_START_SERVER).unwrap();

    let sample_rate: f64 = client.sample_rate() as f64;
    let world = lilv::World::with_load_all();
    let plugin = world
        .plugins()
        .plugin(&world.new_uri("http://drobilla.net/plugins/mda/EPiano"))
        .expect("plugin not found");
    let mut urid_map = lilv::feature::UridMapFeature::default();
    let instance = unsafe {
        plugin
            .instantiate(sample_rate, [urid_map.as_lv2_feature_mut()])
            .unwrap()
    };
    let mut instance = unsafe { instance.activate() };
    let mut ports = Ports::new(world, &plugin, &mut urid_map);

    let mut outputs: Vec<jack::Port<jack::AudioOut>> = (0..ports.output_port_indexes.len())
        .map(|i| {
            let name = format!("output_{}", i);
            client
                .register_port(&name, jack::AudioOut::default())
                .unwrap()
        })
        .collect();
    let input = client
        .register_port("input", jack::MidiIn::default())
        .unwrap();
    let process = move |_: &jack::Client, ps: &jack::ProcessScope| {
        let outputs = outputs.iter_mut().map(|p| p.as_mut_slice(ps));
        unsafe { ports.connect(instance.instance_mut(), outputs, input.iter(ps)) };
        unsafe { instance.run(ps.n_frames() as usize) };
        jack::Control::Continue
    };
    let _active_client = client
        .activate_async((), jack::ClosureProcessHandler::new(process))
        .unwrap();
    std::thread::park();
}

struct Ports {
    output_port_indexes: Vec<usize>,
    midi_port_index: Option<usize>,
    midi_urid: u32,
    midi: LV2AtomSequence,
    control: Vec<(usize, f32)>,
}

impl Ports {
    fn new(
        world: lilv::World,
        plugin: &lilv::plugin::Plugin,
        urid_map: &mut lilv::feature::UridMapFeature,
    ) -> Ports {
        let midi_urid = urid_map.map(
            std::ffi::CStr::from_bytes_with_nul(b"http://lv2plug.in/ns/ext/midi#MidiEvent\0")
                .unwrap(),
        );
        let mut ports = Ports {
            output_port_indexes: Vec::new(),
            midi_port_index: None,
            midi_urid,
            midi: LV2AtomSequence::new(8192),
            control: Vec::new(),
        };
        let audio_port_uri = world.new_uri("http://lv2plug.in/ns/lv2core#AudioPort");
        let input_port_uri = world.new_uri("http://lv2plug.in/ns/lv2core#InputPort");
        let output_port_uri = world.new_uri("http://lv2plug.in/ns/lv2core#OutputPort");
        let control_port_uri = world.new_uri("http://lv2plug.in/ns/lv2core#ControlPort");
        let atom_port_uri = world.new_uri("http://lv2plug.in/ns/ext/atom#AtomPort");
        for (port_index, port) in plugin.iter_ports().enumerate() {
            if port.is_a(&output_port_uri) && port.is_a(&audio_port_uri) {
                ports.output_port_indexes.push(port_index);
            } else if port.is_a(&control_port_uri) && port.is_a(&input_port_uri) {
                let default_value = match port.range().default {
                    Some(v) => v.as_float().unwrap_or(0.0),
                    None => 0.0,
                };
                ports.control.push((port_index, default_value));
            } else if port.is_a(&atom_port_uri) && port.is_a(&input_port_uri) {
                if ports.midi_port_index.is_some() {
                    panic!("Did not expect multiple midi port inputs.");
                }
                ports.midi_port_index = Some(port_index);
            } else {
                panic!("Unhandled port {:?}", port)
            }
        }
        ports
    }

    unsafe fn connect<'a, OS>(
        &mut self,
        instance: &mut lilv::instance::Instance,
        outputs: OS,
        midi: jack::MidiIter,
    ) where
        OS: Iterator<Item = &'a mut [f32]>,
    {
        let mut outputs = outputs.map(|output| output.as_mut_ptr());
        for port_index in self.output_port_indexes.iter() {
            let output = outputs.next().unwrap_or(std::ptr::null_mut());
            instance.connect_port_ptr(*port_index, output);
        }
        if let Some(midi_port_index) = self.midi_port_index {
            instance.connect_port_ptr(midi_port_index, self.midi.as_mut_ptr());
            self.midi.clear();
            for raw_midi in midi {
                let mut event = LV2AtomEvent::new(raw_midi.time as i64, self.midi_urid);
                event.set_size(raw_midi.bytes.len());
                (&mut event.buffer[0..raw_midi.bytes.len()]).copy_from_slice(raw_midi.bytes);
                self.midi.append_event(&event);
            }
        }
        for (port_index, default_value) in self.control.iter_mut() {
            instance.connect_port(*port_index, default_value);
        }
    }
}

/// An atom sequence.
struct LV2AtomSequence {
    buffer: Vec<lv2_raw::LV2AtomSequence>,
}

impl LV2AtomSequence {
    /// Create a new sequence that can hold about desired_capacity bytes.
    fn new(desired_capacity: usize) -> LV2AtomSequence {
        let len = desired_capacity / std::mem::size_of::<lv2_raw::LV2AtomSequence>();
        let mut buffer = Vec::with_capacity(len);
        buffer.resize_with(len, || lv2_raw::LV2AtomSequence {
            atom: lv2_raw::LV2Atom { size: 0, mytype: 0 },
            body: lv2_raw::LV2AtomSequenceBody { unit: 0, pad: 0 },
        });
        let mut seq = LV2AtomSequence { buffer };
        seq.clear();
        seq
    }

    /// Clear all events in the sequence.
    fn clear(&mut self) {
        unsafe { lv2_raw::atomutils::lv2_atom_sequence_clear(self.as_mut_ptr()) }
    }

    /// Append an event to the sequence. If there is no capacity for it, then it will not be
    /// appended.
    fn append_event(&mut self, event: &LV2AtomEvent) {
        unsafe {
            lv2_raw::atomutils::lv2_atom_sequence_append_event(
                self.as_mut_ptr(),
                self.capacity() as u32,
                event.as_ptr(),
            )
        };
    }

    /// Return a mutable pointer to the underlying data.
    fn as_mut_ptr(&mut self) -> *mut lv2_raw::LV2AtomSequence {
        self.buffer.as_mut_ptr()
    }

    /// Get the capacity of the sequence.
    fn capacity(&self) -> usize {
        let slice: &[lv2_raw::LV2AtomSequence] = &self.buffer;
        std::mem::size_of_val(slice)
    }
}

impl std::fmt::Debug for LV2AtomSequence {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let capacity = self.capacity();
        f.debug_struct("Lv2AtomSequence")
            .field("capacity", &capacity)
            .finish()
    }
}

/// The underlying buffer backing the data for an atom event.
type Lv2AtomEventBuffer = [u8; 16];

/// An single atom event.
#[repr(packed)]
struct LV2AtomEvent {
    header: lv2_raw::LV2AtomEvent,
    pub buffer: Lv2AtomEventBuffer,
}

impl LV2AtomEvent {
    /// Create a new atom event with the given time and type. The event can be filled in by setting
    /// the bytes in buffer and calling `set_size`.
    fn new(time_in_frames: i64, my_type: u32) -> LV2AtomEvent {
        let mut event = LV2AtomEvent {
            header: lv2_raw::LV2AtomEvent {
                time_in_frames: 0,
                body: lv2_raw::LV2Atom { size: 0, mytype: 0 },
            },
            buffer: [0; 16],
        };
        event.header.time_in_frames = time_in_frames;
        event.header.body.mytype = my_type;
        event.header.body.size = 0;
        event
    }

    /// Set the size of the atom. Must be less than or equal to the size of the buffer.
    fn set_size(&mut self, size: usize) {
        debug_assert!(size < self.buffer.len(), "{} < {}", size, self.buffer.len());
        self.header.body.size = size as u32;
    }

    /// Return a pointer to the header of the atom.
    #[allow(unaligned_references)]
    fn as_ptr(&self) -> *const lv2_raw::LV2AtomEvent {
        &self.header
    }
}
