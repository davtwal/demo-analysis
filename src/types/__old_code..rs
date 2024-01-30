/// The code in this file is considered old and unused.
/// It's left here in case of necessary re-use and for documentation purposes.

/// BITWRITES and BITREADS
/// Originally, I expected to use ZMQ to pass data between python and rust.
/// However, it was significantly slower than I expected, so I scrapped that idea.
/// Left in its place is all the work I did for making my python structures
/// convertable to byte format.

// types::mod.rs

pub const USIZE_BIT_SIZE: usize = std::mem::size_of::<usize>() * 8;

#[macro_export]
macro_rules! implBitforEnum {
    ($cl:ident) => {
        impl<'a, E: Endianness> BitRead<'a, E> for $cl {
            fn bit_size() -> Option<usize> {
                Some(8)
            }
        
            fn read(stream: &mut bitbuffer::BitReadStream<'a, E>) -> bitbuffer::Result<Self> {
                match stream.read_bytes(1) {
                    Ok(bytes) => Ok($cl::new(bytes[0])),
                    Err(err) => Err(err)
                }
            }
        
            unsafe fn read_unchecked(stream: &mut bitbuffer::BitReadStream<'a, E>, _end: bool) -> bitbuffer::Result<Self> {
                match stream.read_bytes(1) {
                    Ok(bytes) => Ok($cl::new(bytes[0])),
                    Err(err) => Err(err)
                }
            }
        
            fn skip(stream: &mut bitbuffer::BitReadStream<'a, E>) -> bitbuffer::Result<()> {
                stream.read_bytes(1).map(|_| ())
            }
        }

        impl<E: Endianness> BitWrite<E> for $cl {
            fn write(&self, stream: &mut bitbuffer::BitWriteStream<E>) -> bitbuffer::Result<()> {
                stream.write_bytes(&[u8::from(*self)])
            }
        }
    }
}

use bitbuffer::{Endianness, BitWrite, BitWriteStream, BitRead, BitReadStream};
use std::collections::HashMap;

pub(self) fn write_vec<E: Endianness, T: BitWrite<E>>(
    stream: &mut BitWriteStream<E>,
    vec: &Vec<T>
) -> bitbuffer::Result<()> 
{
    stream.write_int(vec.len(), USIZE_BIT_SIZE)?;
    for t in vec {
        stream.write::<T>(&t)?;
    }
    Ok(())
}

pub(self) fn write_hash<E: Endianness, T, U>(
    stream: &mut BitWriteStream<E>,
    map: &HashMap<T, U>
) -> bitbuffer::Result<()> where T: BitWrite<E>, U: BitWrite<E>
{
    stream.write_int(map.len(), USIZE_BIT_SIZE)?;
    for (t, u) in map {
        stream.write::<T>(&t)?;
        stream.write::<U>(&u)?;
    }
    Ok(())
}

pub(self) fn read_vec<'a, T: BitRead<'a, E>, E: Endianness>(
    stream: &mut BitReadStream<'a, E>,
) -> bitbuffer::Result<Vec<T>> {
    let len = stream.read_int(USIZE_BIT_SIZE)?;
    let mut ret = Vec::with_capacity(len);
    for _ in 0..len {
        ret.push(stream.read::<T>()?)
    }
    Ok(ret)
}

pub(self) fn read_hash<'a, T, U, E: Endianness>(
    stream: &mut BitReadStream<'a, E>,
) -> bitbuffer::Result<HashMap<T, U>> 
where 
    T: BitRead<'a, E> + std::cmp::Eq + std::hash::Hash, 
    U: BitRead<'a, E> 
{
    let len = stream.read_int(USIZE_BIT_SIZE)?;
    let mut ret = HashMap::with_capacity(len);
    for _ in 0..len {
        ret.insert(stream.read::<T>()?, stream.read::<U>()?);
    }
    Ok(ret)
}

// types::demo.rs

use super::{write_hash, write_vec, read_hash, read_vec};
use crate::types::USIZE_BIT_SIZE;
use bitbuffer::{BitRead, BitWrite, BitReadSized, BitWriteSized, Endianness};

impl<E: Endianness> BitWrite<E> for TickData {
    fn write(&self, stream: &mut bitbuffer::BitWriteStream<E>) -> bitbuffer::Result<()> {
        stream.write_int(self.players.len(), USIZE_BIT_SIZE)?;
        for pl in &self.players {
            stream.write::<Player>(pl)?;
        }
        stream.write_int(self.buildings.len(), USIZE_BIT_SIZE)?;
        for bl in &self.buildings {
            stream.write_int(*bl.0, 32)?;
            stream.write::<Building>(bl.1)?;
        }
        stream.write_int(u32::from(self.tick), 32)?;
        Ok(())
    }
}

impl<'a, E: Endianness> BitRead<'a, E> for TickData {
    fn bit_size() -> Option<usize> {
        None
    }

    fn read(stream: &mut bitbuffer::BitReadStream<'a, E>) -> bitbuffer::Result<Self> {
        Ok(TickData{
            players: {
                let len = stream.read_int(USIZE_BIT_SIZE)?;
                let mut ret = Vec::with_capacity(len);
                for _ in 0..len {
                    ret.push(stream.read::<Player>()?);
                }
                ret
            },
            buildings: {
                let len = stream.read_int(USIZE_BIT_SIZE)?;
                let mut ret = HashMap::with_capacity(len);
                for _ in 0..len {
                    ret.insert(stream.read_int(32)?, stream.read::<Building>()?);
                }
                ret
            },
            tick: DemoTick::from(stream.read_int::<u32>(32)?)
        })
    }

    unsafe fn read_unchecked(stream: &mut BitReadStream<'a, E>, _end: bool) -> bitbuffer::Result<Self> {
        TickData::read(stream)
    }

    fn skip(stream: &mut BitReadStream<'a, E>) -> bitbuffer::Result<()> {
        TickData::read(stream).map(|_| ())
    }
}

impl<E: Endianness> BitWrite<E> for DemoData {
    fn write(&self, stream: &mut bitbuffer::BitWriteStream<E>) -> bitbuffer::Result<()> {
        stream.write_string(self.demo_filename.to_str().unwrap(), None)?;
        stream.write_string(&self.map_name, None)?;
        stream.write_float(self.duration)?;
        write_vec(stream, &self.rounds)?;
        write_vec(stream, &self.kills)?;
        write_hash(stream, &self.tick_states)?;
        Ok(())
    }
}

impl<'a, E:Endianness> BitRead<'a, E> for DemoData {
    fn bit_size() -> Option<usize> {
        None
    }
    unsafe fn read_unchecked(stream: &mut BitReadStream<'a, E>, _end: bool) -> bitbuffer::Result<Self> {
        Self::read(stream)
    }
    fn skip(stream: &mut BitReadStream<'a, E>) -> bitbuffer::Result<()> {
        Self::read(stream).map(|_|())   
    }

    fn read(stream: &mut BitReadStream<'a, E>) -> bitbuffer::Result<Self> {
        println!("(  ) read::<demodata>");
        Ok(DemoData{
            demo_filename: PathBuf::from(stream.read_string(None)?.to_string()),
            map_name: stream.read_string(None)?.to_string(),
            duration: stream.read_float()?,
            rounds: read_vec::<Round, E>(stream)?,
            kills: read_vec::<Kill, E>(stream)?,
            tick_states: read_hash::<u32, TickData, E>(stream)?,
        })
    }
}

// types::game::entities.rs

impl<E: Endianness> BitWrite<E> for Dispenser {
    fn write(&self, stream: &mut BitWriteStream<E>) -> bitbuffer::Result<()> {
        stream.write_int(self.entity, 32)?;
        stream.write_int(self.builder, 16)?;
        stream.write::<Vector>(&self.position)?;
        stream.write_int(self.level, 8)?;
        stream.write_int(self.max_health, 16)?;
        stream.write_int(self.health, 16)?;
        stream.write_bool(self.building)?;
        stream.write_bool(self.sapped)?;
        stream.write::<Team>(&self.team)?;
        stream.write_float(self.angle)?;

        // dealing with vec<T>
        stream.write_int(self.healing.len(), USIZE_BIT_SIZE)?;
        for x in &self.healing {
            stream.write_int(*x, 16)?;
        }
        stream.write_int(self.metal, 16)?;
        Ok(())
    }
}

impl<'a, E: Endianness> BitRead<'a, E> for Dispenser {
    fn bit_size() -> Option<usize> {
        None
    }

    fn read(stream: &mut BitReadStream<'a, E>) -> bitbuffer::Result<Self> {
        Ok(Dispenser {
            entity: stream.read_int(32)?,
            builder: stream.read_int(16)?,
            position: stream.read::<Vector>()?,
            level: stream.read_int(8)?,
            max_health: stream.read_int(16)?,
            health: stream.read_int(16)?,
            building: stream.read_bool()?,
            sapped: stream.read_bool()?,
            team: stream.read::<Team>()?,
            angle: stream.read_float()?,
            healing: {
                let len = stream.read_int(USIZE_BIT_SIZE)?;
                let mut ret = Vec::with_capacity(len);
                for _ in 0..len {
                    ret.push(stream.read_int(16)?)
                }
                ret
            },
            metal: stream.read_int(16)?
        })
    }

    unsafe fn read_unchecked(stream: &mut BitReadStream<'a, E>, _end: bool) -> bitbuffer::Result<Self> {
        Dispenser::read(stream)
    }

    fn skip(stream: &mut BitReadStream<'a, E>) -> bitbuffer::Result<()> {
        Dispenser::read(stream).map(|_| ())
    }
}

impl<E:Endianness> BitWrite<E> for Building {
    fn write(&self, stream: &mut BitWriteStream<E>) -> bitbuffer::Result<()> {
        match self {
            Building::Sentry(s) => {
                stream.write_int(0, 8)?;
                stream.write::<Sentry>(s)?;
            }
            Building::Dispenser(d) => {
                stream.write_int(1, 8)?;
                stream.write::<Dispenser>(d)?;
            }
            Building::Teleporter(t) => {
                stream.write_int(2, 8)?;
                stream.write::<Teleporter>(t)?;
            }
        }
        Ok(())
    }
}

impl<'a, E: Endianness> BitRead<'a, E> for Building {
    fn bit_size() -> Option<usize> {
        None
    }
    unsafe fn read_unchecked(stream: &mut BitReadStream<'a, E>, _end: bool) -> bitbuffer::Result<Self> {
        Building::read(stream)
    }
    fn skip(stream: &mut BitReadStream<'a, E>) -> bitbuffer::Result<()> {
        Building::read(stream).map(|_| ())
    }
    fn read(stream: &mut BitReadStream<'a, E>) -> bitbuffer::Result<Self> {
        match stream.read_int::<u8>(8)? {
            0 => stream.read::<Sentry>().map(|s| Building::Sentry(s)),
            1 => stream.read::<Dispenser>().map(|d| Building::Dispenser(d)),
            2 => stream.read::<Teleporter>().map(|t| Building::Teleporter(t)),
            _ => Err(bitbuffer::BitError::IndexOutOfBounds { pos: 100, size: 100 })
        }
    }
}

// Python functions that would be used to ask for the data
    #[pyfn(m)]
    fn get_demo_data() -> PyResult<DemoData> {
        let mut data = Vec::new();
        println!("(py) get demo data");
    
        let mut readstrm = send_get(&mut data, u32::MAX)?;

        println!("(py) send_get: {:#?}", readstrm);
        match readstrm.read::<DemoData>() {
            Ok(demodata) => Ok(demodata),
            Err(_) => Err(BIT_ERROR_RET().into())
        }
    }

    #[pyfn(m)]
    fn get_tick_data(tick: u32) -> PyResult<TickData> {
        let mut data = Vec::new();
    
        let mut readstrm = send_get(&mut data, tick)?;
        match readstrm.read::<TickData>() {
            Ok(tickdata) => Ok(tickdata),
            Err(_) => Err(BIT_ERROR_RET().into())
        }
    }

// datatransmit.rs

const ZMQ_SERVER: &'static str = "tcp://localhost:5555";

#[allow(non_snake_case)]
pub fn BIT_ERROR_RET() -> io::Error {
    io::Error::new(
        io::ErrorKind::InvalidData,
        "could not read requested tick"
    )
}

#[allow(non_snake_case)]
pub fn INVALID_INPUT() -> io::Error {
    io::Error::new(
        io::ErrorKind::InvalidInput,
        "invalid request"
    )
}

pub struct DataTransmitter {
    #[allow(dead_code)]
    context: zmq::Context,
    socket: zmq::Socket,
}

impl DataTransmitter {
    pub fn new() -> io::Result<Self> {
        let context = zmq::Context::new();
        let socket = context.socket(zmq::REP)?;

        socket.bind(ZMQ_SERVER)?;

        Ok(DataTransmitter {
            context,
            socket
        })
    }

    /// bool: true if it processed a request, false if no request yet
    /// error if bit writing fails or zmq fails to recv
    pub fn do_any_req<'a, F, T>(&self, get_demodata: F, get_tickdata: T) -> Result<bool, io::Error>
    where
        F: FnOnce() -> Option<&'a DemoData>,
        T: FnOnce(u32) -> Option<&'a TickData> 
    {
        match self.socket.recv_msg(zmq::DONTWAIT) {
            Ok(msg) => {
                println!("msg acquired");
                // Msg should contain a single int
                let req_tick_res = BitReadBuffer::new(
                        msg.to_vec().as_slice(),
                        BigEndian)
                    .read_int::<u32>(0, 32);

                match req_tick_res {
                    Ok(req_tick) => {
                        println!("req tick: {}", req_tick);
                        let mut data = Vec::new();
                        let mut writestr = BitWriteStream::new(&mut data, BigEndian);
                        if req_tick == u32::MAX {
                            if let Some(demodata) = get_demodata() {
                                println!("got demodata");
                                if let Err(_) = writestr.write(demodata) {
                                    println!("error");
                                    let _ = self.socket.send(vec![1 as u8], 0);
                                    return Err(BIT_ERROR_RET());
                                }
                            } else {
                                println!("failed to get demo data");
                                let _ = self.socket.send(vec![1 as u8], 0);
                                //return Ok(true);
                            }
                        } else {
                            if let Some(tickdata) = get_tickdata(req_tick) {
                                if let Err(_) = writestr.write(tickdata) {
                                    let _ = self.socket.send(vec![1 as u8], 0);
                                    return Err(BIT_ERROR_RET());
                                }
                            } else {
                                let _ = self.socket.send(vec![1 as u8], 0);
                                //return Ok(true);
                            }
                        }
                    },
                    Err(_) => {
                        let _ = self.socket.send(vec![1 as u8], 0);
                        return Err(BIT_ERROR_RET());
                    }
                }

                println!("success");
                Ok(true)
            },
            Err(err) => {
                match err {
                    zmq::Error::EAGAIN => Ok(false),
                    _ => Err(io::Error::from(err))
                }
            }
        }
    }
}

// Encapsulates requesting and receiving.
pub fn send_get<'a>(data: &'a mut Vec<u8>, tick: u32) -> io::Result<BitReadStream<'a, BigEndian>> {
    println!("send_get is requesting");
    let context = zmq::Context::new();
    let requester = context.socket(zmq::REQ).unwrap();

    requester.connect(ZMQ_SERVER)?;

    let mut writestr = BitWriteStream::new(data, BigEndian);
    writestr.write_int(tick, bit_size_of::<u32>().unwrap()).unwrap();

    requester.send(data.clone(), 0)?;
    data.clear();

    println!("send get waiting on recv");
    *data = requester.recv_bytes(0)?;

    let mut readstr = BitReadStream::new(BitReadBuffer::new(data, BigEndian));
    if let Ok(status) = readstr.read_int::<u8>(8) {
        println!("status recv'd: {}", status);
        match status {
            0 => Ok(readstr),
            _ => Err(INVALID_INPUT())
        }
    } else {Err(BIT_ERROR_RET())}
}

// app.rs

#[cfg(target_os="windows")]
const PYTHON_EXEC_LOC: &str = ".env/Scripts/python.exe";

#[allow(dead_code)]
pub fn launch_analysis_processes(demodata_vec: &Vec<(PathBuf, DemoData)>) -> io::Result<()> {
    println!("~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~");
    println!("| Starting analysis: \n| - creating transmitter...");
    let transmitter = dt::DataTransmitter::new()?;

    let (tx, rx) = mpsc::channel();

    ctrlc::set_handler(move || {
        println!("ctrl+c spoted");
        tx.send(()).expect("could not send ctrlc signal");
    }).expect("could not set ctrlc handler");

    for (fname, demo_data) in demodata_vec {
        println!("| == Analysing {:#?}", fname.file_name().unwrap());

        let mut py_child = Command::new(PYTHON_EXEC_LOC)
            .current_dir("python")
            .arg("analysis.py")
            .spawn().expect("failed to spawn python");

        loop {
            if let Ok(processed) = transmitter.do_any_req(
                || -> Option<&DemoData> {
                    println!("| Getting demo data...");
                    Some(&demo_data)
                }, |tick: u32| -> Option<&TickData> {
                    demo_data.tick_states.get(&tick)
                }) {
                if processed {break;}
            }
            else {break;}

            if let Ok(_) = rx.try_recv() {
                println!("ctrl+c received");
                py_child.kill().expect("could not kill child");
                break;
            }

            if let Ok(res) = py_child.try_wait() {
                if let Some(_) = res {
                    println!("| !! Exited without requesting data");
                    break;
                }
            } else {
                println!("| Error on try_wait");
                break;
            }
        }

        // loop for either ctrl+c or wait finishes
        loop {
            if let Ok(_) = rx.try_recv() {
                println!("ctrl+received");
                py_child.kill().expect("could not kill child");
                break;
            }

            if let Some(_) = py_child.try_wait().unwrap() {
                println!("process finished");
                break;
            }
        }
    }

    Ok(())
}

// viewing::mod.rs

fn inside_demoviewui_update() {
self.data_transmitter.as_ref().unwrap().do_any_req(|| {
    self.parse_data.as_ref()
}, |tick: u32| {
    if let Some(data) = &self.parse_data {
        data.tick_states.get(&tick)
    } else {None}
}).unwrap();
}

// analysis::data

// //! Contains the data we want to track.

// use std::collections::BTreeMap;

// pub use tf_demo_parser::demo::message::packetentities::EntityId;
// use tf_demo_parser::demo::vector::{Vector, VectorXY};

// /// Data on a per-player basis.
// /// If this data type is acquired from tick analysis, then it this data
// /// is specific to that tick.
// /// Otherwise, if it was acquired from a datasum, then it is considered
// /// the average or total, depending on what the data point is.

// pub struct Distance {
//     pub dist_xy: f32,   // >= 0
//     pub dist_z: f32,    // Real
//     pub dist: f32,      // >= 0
// }

// impl From<(&Vector, &Vector)> for Distance {
//     fn from(value: (&Vector, &Vector)) -> Self {
//         Distance::from_xyz(value.0, value.1)
//     }
// }

// impl From<(&VectorXY, &VectorXY)> for Distance {
//     fn from(value: (&VectorXY, &VectorXY)) -> Self {
//         Distance::from_xy(value.0, value.1)
//     }
// }

// impl Distance {
//     pub fn from_xyz(a: &Vector, b: &Vector) -> Self {
//         let x2 = (b.x - a.x) * (b.x - a.x);
//         let y2 = (b.y - a.y) * (b.y - b.y);
//         let z = b.z - a.z;
//         Distance {
//             dist_xy: f32::sqrt(x2 + y2),
//             dist_z: z,
//             dist: f32::sqrt(x2+y2+z*z)
//         }
//     }

//     pub fn from_xy(a: &VectorXY, b: &VectorXY) -> Self {
//         let d = f32::sqrt((b.x - a.x) * (b.x - a.x) + (b.y - a.y) * (b.y - b.y));
//         Distance {
//             dist_xy: d,
//             dist_z: 0.0,
//             dist: d
//         }
//     }
// }

// pub struct PlayerData {
//     pub id: EntityId,

//     /// Distance to every other player in the game that is currently alive
//     pub dist_to_player: BTreeMap<EntityId, Distance>,
// }

// impl PlayerData {
//     pub fn grouped_with(teammates: &Vec<EntityId>, group_distancing: f32) /*-> Vec<EntityId>*/ {
//         // Grouping is based on XY distance and
//     }

//     pub fn fighting(enemies: &Vec<EntityId>) {

//     }
// }



// /// Data of a single team.
// /// If this data type is acquired from tick analysis, then it this data
// /// is specific to that tick.
// /// Otherwise, if it was acquired from a datasum, then it is considered
// /// the average or total, depending on what the data point is.
// pub struct TeamData {
//     // Data local to each player
//     playerdata: Vec<PlayerData>,

//     // Data that only makes sense when looking at the whole team,
//     // e.g. average distance from medic as a team
//     num_groupings: u8,

//     has_combo: bool,
// }
