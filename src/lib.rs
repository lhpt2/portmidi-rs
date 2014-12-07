#![crate_name = "portmidi"]
#![comment = "PortMidi binding for Rust"]
#![license = "MIT"]
#![crate_type = "lib"]
#![crate_type = "dylib"]

extern crate libc;
extern crate core;
extern crate serialize;

use std::ptr;
use core::mem::transmute;
use libc::c_char;


/**
*    initialize() is the library initialisation function - call this before
*    using the library.
*/
#[inline(never)]
pub fn initialize() -> PmError {
    unsafe {
        PmError::unwrap(ffi::Pm_Initialize())
    }
}


#[deriving(Show, PartialEq, Eq, FromPrimitive)]
pub enum PmError {
    PmNoError = ffi::PmError::PmNoError as int,
    PmGotData = ffi::PmError::PmGotData as int, /* < A "no error" return that also indicates data available */
    PmHostError = ffi::PmError::PmHostError as int,
    PmInvalidDeviceId = ffi::PmError::PmInvalidDeviceId as int, /** out of range or
                        * output device when input is requested or
                        * input device when output is requested or
                        * device is already opened
                        */
    PmInsufficientMemory = ffi::PmError::PmInsufficientMemory as int,
    PmBufferTooSmall = ffi::PmError::PmBufferTooSmall as int,
    PmBufferOverflow = ffi::PmError::PmBufferOverflow as int,
    PmBadPtr = ffi::PmError::PmBadPtr as int, /* PortMidiStream parameter is NULL or
               * stream is not opened or
               * stream is output when input is required or
               * stream is input when output is required */
    PmBadData = ffi::PmError::PmBadData as int, /* illegal midi data, e.g. missing EOX */
    PmInternalError = ffi::PmError::PmInternalError as int,
    PmBufferMaxSize = ffi::PmError::PmBufferMaxSize as int, /* buffer is already as large as it can be */
    /* NOTE: If you add a new error type, be sure to update Pm_GetErrorText() */
}

impl PmError{
  fn unwrap(error: ffi::PmError) -> PmError  {
    FromPrimitive::from_i64(error as i64).unwrap()
  }

  fn wrap(&self) -> ffi::PmError  {
    FromPrimitive::from_i64(*self as i64).unwrap()
  }

}

/**
*   terminate() is the library termination function - call this after
*   using the library.
*/
#[inline(never)]
pub fn terminate() -> PmError {
    unsafe {
        PmError::unwrap(ffi::Pm_Terminate())
    }
}

/**  Translate portmidi error number into human readable message.
*    These strings are constants (set at compile time) so client has
*    no need to allocate storage
*/
#[inline(never)]
pub fn get_error_text(error_code : PmError) -> String {
    unsafe {
        String::from_raw_buf((ffi::Pm_GetErrorText(error_code.wrap()) as *const u8))
    }
}

/**  Translate portmidi host error into human readable message.
    These strings are computed at run time, so client has to allocate storage.
    After this routine executes, the host error is cleared.
*/
#[inline(never)]
pub fn get_host_error_text(msg : *const c_char , len : i32 ) {
    unsafe {
        ffi::Pm_GetHostErrorText(msg, len);
    }
}

pub const HDRLENGTH : i32 = 50;

/* any host error msg will occupy less
than this number of characters */
pub const PM_HOST_ERROR_MSG_LEN : i32 = 256;

/**
    Device enumeration mechanism.

    Device ids range from 0 to Pm_CountDevices()-1.

*/
pub type CPmDeviceID = i32;
pub type PmDeviceID = int;
pub const PM_NO_DEVICE : i32 = -1;

#[deriving(Show)]
pub struct PmDeviceInfo {
    struct_version: int, /* < this internal structure version */
    interf : String, /* < underlying MIDI API, e.g. MMSystem or DirectX */
    pub name : String,    /* < device name, e.g. USB MidiSport 1x1 */
    input : int, /* < true iff input is available */
    output : int, /* < true iff output is available */
    opened : int, /* < used by generic PortMidi code to do error checking on arguments */
}

#[doc(hidden)]
#[repr(C)]
pub struct CPmDeviceInfo {
    struct_version: i32, /* < this internal structure version */
    interf : *const c_char, /* < underlying MIDI API, e.g. MMSystem or DirectX */
    pub name : *const c_char,    /* < device name, e.g. USB MidiSport 1x1 */
    input : i32, /* < true iff input is available */
    output : i32, /* < true iff output is available */
    opened : i32, /* < used by generic PortMidi code to do error checking on arguments */
}

#[doc(hidden)]
impl PmDeviceInfo {
    pub fn wrap(cdevice_info : *const CPmDeviceInfo) -> PmDeviceInfo {
        unsafe {
            PmDeviceInfo {
                struct_version: (*cdevice_info).struct_version as int,
                interf : String::from_raw_buf((*cdevice_info).interf as *const u8),
                name : String::from_raw_buf((*cdevice_info).name as *const u8),
                input : (*cdevice_info).input as int,
                output : (*cdevice_info).output as int,
                opened : (*cdevice_info).opened as int,
            }
        }
    }

    pub fn unwrap(&self) -> CPmDeviceInfo {
        CPmDeviceInfo {
            struct_version: self.struct_version as i32,
            interf :  unsafe { self.interf.to_c_str().into_inner() },
            name :  unsafe { self.name.to_c_str().into_inner() },
            input : self.input as i32,
            output : self.output as i32,
            opened : self.opened as i32,
        }
    }

    pub fn is_input(&self) -> bool {
        self.input > 0
    }

    pub fn is_output(&self) -> bool {
        self.output > 0
    }
}

/**  Get devices count, ids range from 0 to Pm_CountDevices()-1. */
pub fn count_devices() -> int {
    unsafe {
        ffi::Pm_CountDevices() as int
    }
}

/*
    Pm_GetDefaultInputDeviceID(), Pm_GetDefaultOutputDeviceID()

    Return the default device ID or pmNoDevice if there are no devices.
    The result (but not pmNoDevice) can be passed to Pm_OpenMidi().

    The default device can be specified using a small application
    named pmdefaults that is part of the PortMidi distribution. This
    program in turn uses the Java Preferences object created by
    java.util.prefs.Preferences.userRoot().node("/PortMidi"); the
    preference is set by calling
        prefs.put("PM_RECOMMENDED_OUTPUT_DEVICE", prefName);
    or  prefs.put("PM_RECOMMENDED_INPUT_DEVICE", prefName);

    In the statements above, prefName is a string describing the
    MIDI device in the form "interf, name" where interf identifies
    the underlying software system or API used by PortMdi to access
    devices and name is the name of the device. These correspond to
    the interf and name fields of a PmDeviceInfo. (Currently supported
    interfaces are "MMSystem" for Win32, "ALSA" for Linux, and
    "CoreMIDI" for OS X, so in fact, there is no choice of interface.)
    In "interf, name", the strings are actually substrings of
    the full interface and name strings. For example, the preference
    "Core, Sport" will match a device with interface "CoreMIDI"
    and name "In USB MidiSport 1x1". It will also match "CoreMIDI"
    and "In USB MidiSport 2x2". The devices are enumerated in device
    ID order, so the lowest device ID that matches the pattern becomes
    the default device. Finally, if the comma-space (", ") separator
    between interface and name parts of the preference is not found,
    the entire preference string is interpreted as a name, and the
    interface part is the empty string, which matches anything.

    On the MAC, preferences are stored in
      /Users/$NAME/Library/Preferences/com.apple.java.util.prefs.plist
    which is a binary file. In addition to the pmdefaults program,
    there are utilities that can read and edit this preference file.

    On the PC,

    On Linux,

*/
pub fn get_default_input_device_id() -> PmDeviceID {
    unsafe {
        ffi::Pm_GetDefaultInputDeviceID() as int
    }
}

pub fn get_default_output_device_id() -> PmDeviceID {
    unsafe {
        ffi::Pm_GetDefaultOutputDeviceID() as int
    }
}


/**
    Pm_GetDeviceInfo() returns a pointer to a PmDeviceInfo structure
    referring to the device specified by id.
    If id is out of range the function returns NULL.

    The returned structure is owned by the PortMidi implementation and must
    not be manipulated or freed. The pointer is guaranteed to be valid
    between calls to Pm_Initialize() and Pm_Terminate().
*/
#[inline(never)]
pub fn get_device_info(device : PmDeviceID) -> Option<PmDeviceInfo> {
    let c_info = unsafe { ffi::Pm_GetDeviceInfo(device as i32) };
    if c_info.is_null() {
        None
    }
    else {
        Some(PmDeviceInfo::wrap(c_info))
    }
}

#[deriving(Clone, PartialEq, Eq, Decodable, Encodable, Show)]
pub struct PmMessage { /**< see PmEvent */
    pub status : i8,
    pub data1 : i8,
    pub data2 : i8,
}

/**
    Pm_Message() encodes a short Midi message into a 32-bit word. If data1
    and/or data2 are not present, use zero.

    Pm_MessageStatus(), Pm_MessageData1(), and
    Pm_MessageData2() extract fields from a 32-bit midi message.
*/
#[doc(hidden)]
impl PmMessage {
    pub fn wrap(cmessage : ffi::CPmMessage) -> PmMessage {
        PmMessage {
            status:  ((cmessage) & 0xFF) as i8,
            data1 : (((cmessage) >> 8) & 0xFF) as i8,
            data2 : (((cmessage) >> 16) & 0xFF) as i8,
        }
    }

    pub fn unwrap(&self) -> ffi::CPmMessage {
        ((((self.data2 as i32) << 16) & 0xFF0000) |
          (((self.data1 as i32) << 8) & 0xFF00) |
          ((self.status as i32) & 0xFF)) as i32
    }
}


/**
   All midi data comes in the form of PmEvent structures. A sysex
   message is encoded as a sequence of PmEvent structures, with each
   structure carrying 4 bytes of the message, i.e. only the first
   PmEvent carries the status byte.

   Note that MIDI allows nested messages: the so-called "real-time" MIDI
   messages can be inserted into the MIDI byte stream at any location,
   including within a sysex message. MIDI real-time messages are one-byte
   messages used mainly for timing (see the MIDI spec). PortMidi retains
   the order of non-real-time MIDI messages on both input and output, but
   it does not specify exactly how real-time messages are processed. This
   is particulary problematic for MIDI input, because the input parser
   must either prepare to buffer an unlimited number of sysex message
   bytes or to buffer an unlimited number of real-time messages that
   arrive embedded in a long sysex message. To simplify things, the input
   parser is allowed to pass real-time MIDI messages embedded within a
   sysex message, and it is up to the client to detect, process, and
   remove these messages as they arrive.

   When receiving sysex messages, the sysex message is terminated
   by either an EOX status byte (anywhere in the 4 byte messages) or
   by a non-real-time status byte in the low order byte of the message.
   If you get a non-real-time status byte but there was no EOX byte, it
   means the sysex message was somehow truncated. This is not
   considered an error; e.g., a missing EOX can result from the user
   disconnecting a MIDI cable during sysex transmission.

   A real-time message can occur within a sysex message. A real-time
   message will always occupy a full PmEvent with the status byte in
   the low-order byte of the PmEvent message field. (This implies that
   the byte-order of sysex bytes and real-time message bytes may not
   be preserved -- for example, if a real-time message arrives after
   3 bytes of a sysex message, the real-time message will be delivered
   first. The first word of the sysex message will be delivered only
   after the 4th byte arrives, filling the 4-byte PmEvent message field.

   The timestamp field is observed when the output port is opened with
   a non-zero latency. A timestamp of zero means "use the current time",
   which in turn means to deliver the message with a delay of
   latency (the latency parameter used when opening the output port.)
   Do not expect PortMidi to sort data according to timestamps --
   messages should be sent in the correct order, and timestamps MUST
   be non-decreasing. See also "Example" for Pm_OpenOutput() above.

   A sysex message will generally fill many PmEvent structures. On
   output to a PortMidiStream with non-zero latency, the first timestamp
   on sysex message data will determine the time to begin sending the
   message. PortMidi implementations may ignore timestamps for the
   remainder of the sysex message.

   On input, the timestamp ideally denotes the arrival time of the
   status byte of the message. The first timestamp on sysex message
   data will be valid. Subsequent timestamps may denote
   when message bytes were actually received, or they may be simply
   copies of the first timestamp.

   Timestamps for nested messages: If a real-time message arrives in
   the middle of some other message, it is enqueued immediately with
   the timestamp corresponding to its arrival time. The interrupted
   non-real-time message or 4-byte packet of sysex data will be enqueued
   later. The timestamp of interrupted data will be equal to that of
   the interrupting real-time message to insure that timestamps are
   non-decreasing.
 */
#[deriving(Clone, PartialEq, Eq, Decodable, Encodable, Show)]
pub  struct PmEvent {
    pub message : PmMessage,
    pub timestamp : ffi::CPmTimestamp,
}

#[doc(hidden)]
impl PmEvent {
    pub fn wrap(cevent : ffi::CPmEvent) -> PmEvent {
        PmEvent {
            message:  PmMessage::wrap(cevent.message),
            timestamp : cevent.timestamp,
        }
    }

    pub fn unwrap(&self) -> ffi::CPmEvent {
        ffi::CPmEvent {
            message:  self.message.unwrap(),
            timestamp : self.timestamp,
        }
    }
}


/// Representation of an input midi port.
pub struct PmInputPort {
    c_pm_stream : *const ffi::CPortMidiStream,
    input_device : CPmDeviceID,
    buffer_size : i32,
}

impl PmInputPort {
    /**
    * Constructor for PmInputPort.
    *
    * Return a new PmInputPort.
    */
    pub fn new(input_device : PmDeviceID, buffer_size: int) -> PmInputPort {
        PmInputPort {
            c_pm_stream : ptr::null(),
            input_device : input_device as i32,
            buffer_size : buffer_size as i32,
        }
    }

    #[inline(never)]
    pub fn open(&mut self)  -> PmError {
        unsafe {
            PmError::unwrap(ffi::Pm_OpenInput(&self.c_pm_stream, self.input_device, ptr::null(), self.buffer_size, ptr::null(), ptr::null()))
        }
    }

    /**
    *    Test whether stream has a pending host error. Normally, the client finds
    *    out about errors through returned error codes, but some errors can occur
    *    asynchronously where the client does not
    *    explicitly call a function, and therefore cannot receive an error code.
    *    The client can test for a pending error using has_host_error(). If true,
    *    the error can be accessed and cleared by calling get_Error_text().
    *    Errors are also cleared by calling other functions that can return
    *    errors, e.g. open_input(), open_output(), read(), write(). The
    *    client does not need to call Pm_HasHostError(). Any pending error will be
    *    reported the next time the client performs an explicit function call on
    *    the stream, e.g. an input or output operation. Until the error is cleared,
    *    no new error codes will be obtained, even for a different stream.
    */
    #[inline(never)]
    pub fn has_host_error(&self) -> i32  {
        unsafe {
            ffi::Pm_HasHostError(self.c_pm_stream)
        }

    }

    /**
        Read one midi note.
        Retur the note event if available or Err(pmNoError) if no midi event is avaible or Err() if an error occurs.

        Pm_Read() retrieves midi data into a buffer, and returns the number
        of events read. Result is a non-negative number unless an error occurs,
        in which case a PmError value will be returned.

        Buffer Overflow

        The problem: if an input overflow occurs, data will be lost, ultimately
        because there is no flow control all the way back to the data source.
        When data is lost, the receiver should be notified and some sort of
        graceful recovery should take place, e.g. you shouldn't resume receiving
        in the middle of a long sysex message.

        With a lock-free fifo, which is pretty much what we're stuck with to
        enable portability to the Mac, it's tricky for the producer and consumer
        to synchronously reset the buffer and resume normal operation.

        Solution: the buffer managed by PortMidi will be flushed when an overflow
        occurs. The consumer (Pm_Read()) gets an error message (pmBufferOverflow)
        and ordinary processing resumes as soon as a new message arrives. The
        remainder of a partial sysex message is not considered to be a "new
        message" and will be flushed as well.

    */
    pub fn read(&mut self) -> Result<PmEvent, PmError> {

        //get one note a the time
         let mut pevent : ffi::CPmEvent = ffi::CPmEvent {
            message : 0,
            timestamp : 0,
        };
//        println!("portmidi::midi before read In stream:{:?}", self.c_pm_stream);
        let nbnote : i16 = unsafe {
            ffi::Pm_Read(self.c_pm_stream, &mut pevent, 1)
        };
//        println!("portmidi::midi after read");
        match nbnote {
            y if y == 0 => Err(PmError::PmNoError),
            y if y > 0 => Ok(PmEvent::wrap(pevent)),
            _ => Err(unsafe { transmute::<i16, PmError>(nbnote) })
        }
    }

    /**
        Pm_Poll() tests whether input is available,
        returning pmGotData, pmNoError, or an error value.
    */
    pub fn poll(&self)  -> PmError  {
        unsafe {
            PmError::unwrap(ffi::Pm_Poll(self.c_pm_stream))
        }
    }

    /**
        Pm_Close() closes a midi stream, flushing any pending buffers.
        (PortMidi attempts to close open streams when the application
        exits -- this is particularly difficult under Windows.)
    */
    pub fn close(&mut self)  -> PmError  {
//      println!("portmidi::midi inport close");
        unsafe {
            PmError::unwrap(ffi::Pm_Close(self.c_pm_stream))
        }
    }
}


/// Representation of an output midi port.
pub struct PmOutputPort {
    c_pm_stream : *const ffi::CPortMidiStream,
    output_device : CPmDeviceID,
    buffer_size : i32,
}

impl PmOutputPort {
    /**
    * Constructor for PmOutputPort.
    *
    * Return a new PmOutputPort.
    */
    pub fn new(output_device : PmDeviceID, buffer_size: int) -> PmOutputPort {
        PmOutputPort {
            c_pm_stream : ptr::null(),
            output_device : output_device as i32,
            buffer_size : buffer_size as i32,
        }
    }

    #[inline(never)]
    pub fn open(&mut self)  -> PmError {

        unsafe {
            PmError::unwrap(ffi::Pm_OpenOutput(&self.c_pm_stream, self.output_device, ptr::null(), self.buffer_size, ptr::null(), ptr::null(), 0))
        }
    }

    /**
    *    Test whether stream has a pending host error. Normally, the client finds
    *    out about errors through returned error codes, but some errors can occur
    *    asynchronously where the client does not
    *    explicitly call a function, and therefore cannot receive an error code.
    *    The client can test for a pending error using has_host_error(). If true,
    *    the error can be accessed and cleared by calling get_Error_text().
    *    Errors are also cleared by calling other functions that can return
    *    errors, e.g. open_input(), open_output(), read(), write(). The
    *    client does not need to call Pm_HasHostError(). Any pending error will be
    *    reported the next time the client performs an explicit function call on
    *    the stream, e.g. an input or output operation. Until the error is cleared,
    *    no new error codes will be obtained, even for a different stream.
    */
    #[inline(never)]
    pub fn has_host_error(&self) -> i32  {
        unsafe {
            ffi::Pm_HasHostError(self.c_pm_stream)
        }

    }

    /**
        Pm_Abort() terminates outgoing messages immediately
        The caller should immediately close the output port;
        this call may result in transmission of a partial midi message.
        There is no abort for Midi input because the user can simply
        ignore messages in the buffer and close an input device at
        any time.
     */
    pub fn abort(&mut self) -> PmError {
        unsafe {
            PmError::unwrap(ffi::Pm_Abort(self.c_pm_stream))
        }
    }

    /**
        Pm_Close() closes a midi stream, flushing any pending buffers.
        (PortMidi attempts to close open streams when the application
        exits -- this is particularly difficult under Windows.)
    */
    pub fn close(&mut self)  -> PmError  {
        unsafe {
            PmError::unwrap(ffi::Pm_Close(self.c_pm_stream))
        }
    }

    /**
        Pm_Write() writes midi data from a buffer. This may contain:
            - short messages
        or
            - sysex messages that are converted into a sequence of PmEvent
              structures, e.g. sending data from a file or forwarding them
              from midi input.

        Use Pm_WriteSysEx() to write a sysex message stored as a contiguous
        array of bytes.

        Sysex data may contain embedded real-time messages.
    */
    pub fn write_event(&mut self, midievent : PmEvent)  -> PmError  {
        let cevent : ffi::CPmEvent = midievent.unwrap();
        unsafe {
            PmError::unwrap(ffi::Pm_Write(self.c_pm_stream, &cevent, 1))
        }
    }

    /**
        Pm_WriteShort() writes a timestamped non-system-exclusive midi message.
        Messages are delivered in order as received, and timestamps must be
        non-decreasing. (But timestamps are ignored if the stream was opened
        with latency = 0.)
    */
    pub fn write_message(&mut self, midimessage : PmMessage)  -> PmError  {
        let cevent : ffi::CPmMessage = midimessage.unwrap();
        unsafe {
            PmError::unwrap(ffi::Pm_WriteShort(self.c_pm_stream, 0, cevent))
        }
    }
}


mod ffi {
    use libc::{c_char, c_void};

    pub type CPortMidiStream = c_void;

    #[doc(hidden)]
    pub type CPmMessage = i32;

    pub type CPmTimestamp = u32;

    #[doc(hidden)]
    #[repr(C)]
    pub struct CPmEvent {
        pub message : CPmMessage,
        pub timestamp : CPmTimestamp,
    }

    #[deriving(Show, FromPrimitive)]
    #[repr(C)]
    pub enum PmError {
        PmNoError = 0,
        PmGotData = 1, /* < A "no error" return that also indicates data available */
        PmHostError = -10000,
        PmInvalidDeviceId = -9999, /** out of range or
                            * output device when input is requested or
                            * input device when output is requested or
                            * device is already opened
                            */
        PmInsufficientMemory = -9998,
        PmBufferTooSmall = -9997,
        PmBufferOverflow = -9996,
        PmBadPtr = -9995, /* PortMidiStream parameter is NULL or
                   * stream is not opened or
                   * stream is output when input is required or
                   * stream is input when output is required */
        PmBadData = -9994, /* illegal midi data, e.g. missing EOX */
        PmInternalError = -9993,
        PmBufferMaxSize = -9992, /* buffer is already as large as it can be */
    }

    #[link(name = "portmidi")]
    extern "C" {
        pub fn Pm_Initialize() -> PmError;
        pub fn Pm_Terminate()-> PmError;
        pub fn Pm_HasHostError(stream : *const CPortMidiStream ) -> i32;
        pub fn Pm_GetErrorText(errorCode : PmError) -> *const c_char;
        pub fn Pm_GetHostErrorText(msg : *const c_char , len : i32 );
        pub fn Pm_CountDevices() -> u32;
        pub fn Pm_GetDefaultInputDeviceID() -> super::CPmDeviceID;
        pub fn Pm_GetDefaultOutputDeviceID() -> super::CPmDeviceID;
        pub fn Pm_GetDeviceInfo(id:super::CPmDeviceID) -> *const super::CPmDeviceInfo;
        pub fn Pm_OpenInput(stream: *const *const CPortMidiStream, inputDevice : super::CPmDeviceID, inputDriverInfo: *const c_void, bufferSize : i32, time_proc: *const c_void, time_info: *const c_void) -> PmError;
        pub fn Pm_OpenOutput(stream : *const *const CPortMidiStream, outputDevice : super::CPmDeviceID, inputDriverInfo: *const c_void, bufferSize : i32, time_proc: *const c_void, time_info: *const c_void, latency:i32) -> PmError;
        pub fn Pm_Read(stream : *const CPortMidiStream, buffer : *mut CPmEvent , length : i32) -> i16;
        pub fn Pm_Abort(stream : *const CPortMidiStream) -> PmError;
        pub fn Pm_Close(stream : *const CPortMidiStream) -> PmError;
        pub fn Pm_Poll(stream : *const CPortMidiStream) -> PmError;
        pub fn Pm_Write(stream : *const CPortMidiStream, buffer : *const CPmEvent , length : i32) -> PmError;
        pub fn Pm_WriteShort(stream : *const CPortMidiStream, timestamp : CPmTimestamp , message : CPmMessage) -> PmError;
    }
}


#[cfg(test)]
mod tests {
    #[test]
    fn test_midiin() {
        let error:super::PmError = super::initialize();
        assert_eq!(error as int, super::PmError::PmNoError as int);

        let nbdevice : int = super::count_devices();
        println!("portmidi nb device {}", nbdevice);
        let defdevin : int = super::get_default_input_device_id();
        println!("portmidi default input device {}", defdevin);
        let defdevout : int = super::get_default_output_device_id();
        println!("portmidi default output device {}", defdevout);

        let ininfo = super::get_device_info(defdevin);
        println!("portmidi default input device info {}", ininfo);

        let outinfo = super::get_device_info(defdevout);
        println!("portmidi default output device info {}", outinfo);

        let mut inport : super::PmInputPort = super::PmInputPort::new(defdevin, 0);
        let inerror = inport.open();
        assert_eq!(inerror as int, super::PmError::PmNoError as int);

        let mut outport : super::PmOutputPort = super::PmOutputPort::new(defdevout, 100);
        let outerror = outport.open();
        println!("portmidi new output device {}", outerror);
        assert_eq!(outerror as int, super::PmError::PmNoError as int);

        let read_midi = inport.read();
        println!("portmidi input note {}", read_midi);
        match read_midi    {
            Ok(notes) => println!("portmidi read midi note {}", notes),
            Err(super::PmError::PmNoError) => println!("portmidi read midi no note {}", super::PmError::PmNoError),
            Err(err) => println!("portmidi read midi error {}", err)
        }

        let innote = inport.poll();
        assert_eq!(innote as int, super::PmError::PmNoError as int);

        //send note
        let note1 = super::PmEvent {
            message : super::PmMessage {
                status : 1 | 0x90, //chanell and note on
                data1 : 36, //note number
                data2 : 90, // velocity
            },
            timestamp : 0
        };
        let sendnoteerr = outport.write_event(note1);
        assert_eq!(sendnoteerr as int, super::PmError::PmNoError as int);

        let note2 = super::PmMessage {
            status : 1 | 0x80, //chanell and note off
            data1 : 36, //note number
            data2 : 0, // velocity
        };
        let sendnote2err = outport.write_message(note2);
        assert_eq!(sendnote2err as int, super::PmError::PmNoError as int);

        //close out port
        let aborterr = outport.abort();
        assert_eq!(aborterr as int, super::PmError::PmNoError as int);
        let outcloseerr = outport.close();
        assert_eq!(outcloseerr as int, super::PmError::PmNoError as int);

        //close in port
        let incloseerr = inport.close();
        assert_eq!(incloseerr as int, super::PmError::PmNoError as int);

        //terminate midi
        let error:super::PmError = super::terminate();
        assert_eq!(error as int, super::PmError::PmNoError as int);
    }
}

