#[doc(hidden)]
pub extern crate libc;
#[doc(hidden)]
pub extern crate libretro_sys;

use std::mem;
use std::ptr;
use std::slice;
use std::ffi::{CStr, CString};
use std::cmp::max;

pub use libretro_sys::{PixelFormat, Region};

pub struct CoreInfo {
    library_name: CString,
    library_version: CString,
    supported_romfile_extensions: CString,
    require_path_when_loading_roms: bool,
    allow_frontend_to_extract_archives: bool
}

impl CoreInfo {
    pub fn new( name: &str, version: &str ) -> CoreInfo {
        CoreInfo {
            library_name: CString::new( name ).unwrap(),
            library_version: CString::new( version ).unwrap(),
            supported_romfile_extensions: CString::new( "" ).unwrap(),
            require_path_when_loading_roms: false,
            allow_frontend_to_extract_archives: true
        }
    }

    pub fn supports_roms_with_extension( mut self, mut extension: &str ) -> Self {
        if extension.starts_with( "." ) {
            extension = &extension[ 1.. ];
        }

        let mut string = CString::new( "" ).unwrap();
        mem::swap( &mut string, &mut self.supported_romfile_extensions );

        let mut vec = string.into_bytes();
        if vec.is_empty() == false {
            vec.push( '|' as u8 );
        }

        vec.extend_from_slice( extension.as_bytes() );

        match extension {
            "gz" | "xz"  |
            "zip" | "rar" | "7z" | "tar" | "tgz" | "txz" | "bz2" |
            "tar.gz" | "tar.bz2"| "tar.xz" => {
                self.allow_frontend_to_extract_archives = false;
            },
            _ => {}
        }

        self.supported_romfile_extensions = CString::new( vec ).unwrap();
        self
    }

    pub fn requires_path_when_loading_roms( mut self ) -> Self {
        self.require_path_when_loading_roms = true;
        self
    }
}

pub struct AudioVideoInfo {
    width: u32,
    height: u32,
    max_width: u32,
    max_height: u32,
    frames_per_second: f64,
    audio_sample_rate: f64,
    aspect_ratio: Option< f32 >,
    pixel_format: PixelFormat,
    game_region: Option< Region >
}

impl AudioVideoInfo {
    pub fn new() -> AudioVideoInfo {
        AudioVideoInfo {
            width: 0,
            height: 0,
            max_width: 0,
            max_height: 0,
            frames_per_second: 0.0,
            aspect_ratio: None,
            pixel_format: PixelFormat::RGB565,
            audio_sample_rate: 0.0,
            game_region: None
        }
    }

    pub fn video( mut self, width: u32, height: u32, frames_per_second: f64, pixel_format: PixelFormat ) -> Self {
        self.width = width;
        self.height = height;
        self.max_width = max( self.max_width, width );
        self.max_height = max( self.max_height, height );
        self.frames_per_second = frames_per_second;
        self.pixel_format = pixel_format;
        self
    }

    pub fn max_video_size( mut self, max_width: u32, max_height: u32 ) -> Self {
        self.max_width = max( self.max_width, max_width );
        self.max_height = max( self.max_height, max_height );
        self
    }

    pub fn aspect_ratio( mut self, aspect_ratio: f32 ) -> Self {
        self.aspect_ratio = Some( aspect_ratio );
        self
    }

    pub fn audio( mut self, sample_rate: f64 ) -> Self {
        self.audio_sample_rate = sample_rate;
        self
    }

    pub fn region( mut self, game_region: Region ) -> Self {
        self.game_region = Some( game_region );
        self
    }

    fn infer_game_region( &self ) -> Region {
        self.game_region.unwrap_or_else( || {
            if self.frames_per_second > 59.0 {
                Region::NTSC
            } else {
                Region::PAL
            }
        })
    }
}

pub struct GameData {
    path: Option< String >,

    // The 'static lifetime here is a lie, but it's safe anyway
    // since the user doesn't get direct access to this reference,
    // and he has to give us a GameData object back in on_unload_game,
    // and since we're the only source of those he has to give us
    // the one that he got in on_load_game.
    data: Option< &'static [u8] >
}

impl GameData {
    pub fn path( &self ) -> Option< &str > {
        self.path.as_ref().map( |path| &path[..] )
    }

    pub fn data( &self ) -> Option< &[u8] > {
        self.data.map( |data| data as &[u8] )
    }

    pub fn is_empty( &self ) -> bool {
        self.path().is_none() && self.data().is_none()
    }
}

pub enum LoadGameResult {
    Success( AudioVideoInfo ),
    Failed( GameData )
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub enum JoypadButton {
    A,
    B,
    X,
    Y,
    Select,
    Start,
    Up,
    Down,
    Left,
    Right,
    L1,
    L2,
    L3,
    R1,
    R2,
    R3
}

pub trait Core: Default {
    fn info() -> CoreInfo;
    fn on_load_game( &mut self, game_data: GameData ) -> LoadGameResult;
    fn on_unload_game( &mut self ) -> GameData;
    fn on_run( &mut self, handle: &mut RuntimeHandle );
    fn on_reset( &mut self );
    fn save_memory( &mut self ) -> Option< &mut [u8] > {
        None
    }
    fn rtc_memory( &mut self ) -> Option< &mut [u8] > {
        None
    }
    fn system_memory( &mut self ) -> Option< &mut [u8] > {
        None
    }
    fn video_memory( &mut self ) -> Option< &mut [u8] > {
        None
    }
}

#[inline]
#[doc(hidden)]
unsafe fn call_environment< T >( command: libc::c_uint, pointer: &T ) -> Result< (), () > {
    let ok = ENVIRONMENT_CALLBACK.unwrap()( command, mem::transmute( pointer ) );
    if ok {
        Ok(())
    } else {
        Err(())
    }
}

/// Safe wrappers around libretro enviornment commands
pub mod environment {
    use super::*;

    /// Wrapper for RETRO_ENVIRONMENT_GET_SYSTEM_DIRECTORY
    /// Returns an owned rust String
    pub fn get_system_directory() -> Option<String> {
        let ptr: *const libc::c_char = std::ptr::null();
        unsafe { call_environment( libretro_sys::ENVIRONMENT_GET_SYSTEM_DIRECTORY, &ptr ).expect("ENVIRONMENT_GET_SYSTEM_DIRECTORY failed") };
        unsafe { CStr::from_ptr( ptr ).to_str().ok().map( |path| path.to_owned() ) }
    }

}

static mut ENVIRONMENT_CALLBACK: Option< libretro_sys::EnvironmentFn > = None;

#[doc(hidden)]
pub struct Retro< B: Core > {
    video_refresh_callback: Option< libretro_sys::VideoRefreshFn >,
    audio_sample_callback: Option< libretro_sys::AudioSampleFn >,
    audio_sample_batch_callback: Option< libretro_sys::AudioSampleBatchFn >,
    input_poll_callback: Option< libretro_sys::InputPollFn >,
    input_state_callback: Option< libretro_sys::InputStateFn >,

    core: B,

    is_game_loaded: bool,
    av_info: AudioVideoInfo,
    total_audio_samples_uploaded: usize
}

macro_rules! set_callback {
    ($output: expr, $input: expr) => (
        unsafe {
            if $input == mem::transmute( 0 as usize ) {
                $output = None;
            } else {
                $output = Some( $input );
            }
        }
    )
}

impl< B: Core > Retro< B > {
    fn new( core: B ) -> Self {
        Retro {
            video_refresh_callback: None,
            audio_sample_callback: None,
            audio_sample_batch_callback: None,
            input_poll_callback: None,
            input_state_callback: None,

            core: core,

            is_game_loaded: false,
            av_info: AudioVideoInfo::new(),
            total_audio_samples_uploaded: 0
        }
    }

    #[must_use]
    unsafe fn call_environment< T >( &mut self, command: libc::c_uint, pointer: &T ) -> Result< (), () > {
        call_environment(command, pointer)
    }

    pub fn on_get_system_info( info: *mut libretro_sys::SystemInfo ) {
        assert_ne!( info, ptr::null_mut() );
        let info = unsafe { &mut *info };

        // Pointers in SystemInfo have to be statically allocated,
        // which is why we do this.
        static mut INFO: Option< *const CoreInfo > = None;
        let core_info = unsafe {
            if INFO.is_none() {
                INFO = Some( Box::into_raw( Box::new( B::info() ) ) );
            }
            INFO.map( |core_info| &*core_info ).unwrap()
        };

        info.library_name = core_info.library_name.as_ptr();
        info.library_version = core_info.library_version.as_ptr();
        info.valid_extensions = core_info.supported_romfile_extensions.as_ptr();
        info.need_fullpath = core_info.require_path_when_loading_roms;
        info.block_extract = core_info.allow_frontend_to_extract_archives == false;
    }

    pub fn on_set_environment( callback: libretro_sys::EnvironmentFn ) {
        set_callback!( ENVIRONMENT_CALLBACK, callback );
    }

    pub fn on_set_video_refresh( &mut self, callback: libretro_sys::VideoRefreshFn ) {
        set_callback!( self.video_refresh_callback, callback );
    }

    pub fn on_set_audio_sample( &mut self, callback: libretro_sys::AudioSampleFn ) {
        set_callback!( self.audio_sample_callback, callback );
    }

    pub fn on_set_audio_sample_batch( &mut self, callback: libretro_sys::AudioSampleBatchFn ) {
        set_callback!( self.audio_sample_batch_callback, callback );
    }

    pub fn on_set_input_poll( &mut self, callback: libretro_sys::InputPollFn ) {
        set_callback!( self.input_poll_callback, callback );
    }

    pub fn on_set_input_state( &mut self, callback: libretro_sys::InputStateFn ) {
        set_callback!( self.input_state_callback, callback );
    }

    pub fn on_get_system_av_info( &mut self, info: *mut libretro_sys::SystemAvInfo ) {
        assert_ne!( info, ptr::null_mut() );
        let info = unsafe { &mut *info };

        info.geometry.base_width = self.av_info.width as libc::c_uint;
        info.geometry.base_height = self.av_info.height as libc::c_uint;
        info.geometry.max_width = self.av_info.max_width as libc::c_uint;
        info.geometry.max_height = self.av_info.max_height as libc::c_uint;
        info.geometry.aspect_ratio = self.av_info.aspect_ratio.unwrap_or( 0.0 );
        info.timing.fps = self.av_info.frames_per_second;
        info.timing.sample_rate = self.av_info.audio_sample_rate;
    }

    pub fn on_set_controller_port_device( &mut self, _port: libc::c_uint, _device: libc::c_uint ) {
    }

    pub fn on_reset( &mut self ) {
        self.core.on_reset();
    }

    pub fn on_load_game( &mut self, game_info: *const libretro_sys::GameInfo ) -> bool {
        assert_eq!( self.is_game_loaded, false );

        let game_info = if game_info == ptr::null() {
            None
        } else {
            Some( unsafe { &*game_info } )
        };

        let game_data = match game_info {
            Some( game_info ) => {
                let path = if game_info.path == ptr::null() {
                    None
                } else {
                    unsafe {
                        CStr::from_ptr( game_info.path ).to_str().ok().map( |path| path.to_owned() )
                    }
                };

                let data = if game_info.data == ptr::null() && game_info.size == 0 {
                    None
                } else {
                    unsafe {
                        Some( slice::from_raw_parts( game_info.data as *const u8, game_info.size ) )
                    }
                };

                GameData {
                    path: path,
                    data: data
                }
            },
            None => {
                GameData {
                    path: None,
                    data: None
                }
            }
        };

        let result = self.core.on_load_game( game_data );
        match result {
            LoadGameResult::Success( av_info ) => {
                self.av_info = av_info;
                unsafe {
                    let pixel_format = self.av_info.pixel_format;
                    self.call_environment( libretro_sys::ENVIRONMENT_SET_PIXEL_FORMAT, &pixel_format ).unwrap();
                }

                self.is_game_loaded = true;
                true
            },
            LoadGameResult::Failed( _ ) => false
        }
    }

    pub fn on_load_game_special( &mut self, _game_type: libc::c_uint, _info: *const libretro_sys::GameInfo, _num_info: libc::size_t ) -> bool {
        false
    }

    pub fn on_run( &mut self ) {
        let mut handle = RuntimeHandle {
            video_refresh_callback: self.video_refresh_callback.unwrap(),
            input_state_callback: self.input_state_callback.unwrap(),
            audio_sample_batch_callback: self.audio_sample_batch_callback.unwrap(),
            upload_video_frame_already_called: false,
            audio_samples_uploaded: 0,

            video_width: self.av_info.width,
            video_height: self.av_info.height,
            video_frame_bytes_per_pixel: match self.av_info.pixel_format {
                PixelFormat::ARGB1555 | PixelFormat::RGB565 => 2,
                PixelFormat::ARGB8888 => 4
            }
        };

        unsafe {
            self.input_poll_callback.unwrap()();
        }

        self.core.on_run( &mut handle );

        self.total_audio_samples_uploaded += handle.audio_samples_uploaded;
        let required_audio_sample_count_per_frame = (self.av_info.audio_sample_rate / self.av_info.frames_per_second) * 2.0;
        assert!(
            self.total_audio_samples_uploaded as f64 >= required_audio_sample_count_per_frame,
            format!( "You need to upload at least {} audio samples each frame!", required_audio_sample_count_per_frame )
        );

        self.total_audio_samples_uploaded -= required_audio_sample_count_per_frame as usize;
    }

    pub fn on_serialize_size( &mut self ) -> libc::size_t {
        0
    }

    pub fn on_serialize( &mut self, _data: *mut libc::c_void, _size: libc::size_t ) -> bool {
        false
    }

    pub fn on_unserialize( &mut self, _data: *const libc::c_void, _size: libc::size_t ) -> bool {
        false
    }

    pub fn on_cheat_reset( &mut self ) {
    }

    pub fn on_cheat_set( &mut self, _index: libc::c_uint, _is_enabled: bool, _code: *const libc::c_char ) {
    }

    pub fn on_unload_game( &mut self ) {
        if self.is_game_loaded == false {
            return;
        }

        let _ = self.core.on_unload_game();
    }

    pub fn on_get_region( &mut self ) -> libc::c_uint {
        self.av_info.infer_game_region().to_uint()
    }

    fn memory_data( &mut self, id: libc::c_uint ) -> Option< &mut [u8] > {
        match id {
            libretro_sys::MEMORY_SAVE_RAM => self.core.save_memory(),
            libretro_sys::MEMORY_RTC => self.core.rtc_memory(),
            libretro_sys::MEMORY_SYSTEM_RAM => self.core.system_memory(),
            libretro_sys::MEMORY_VIDEO_RAM => self.core.video_memory(),
            _ => unreachable!(),
        }
    }

    pub fn on_get_memory_data( &mut self, id: libc::c_uint ) -> *mut libc::c_void {
        self.memory_data( id )
            .map( |d| d as *mut _ as *mut libc::c_void )
            .unwrap_or( ptr::null_mut() )
    }

    pub fn on_get_memory_size( &mut self, id: libc::c_uint ) -> libc::size_t {
        self.memory_data( id )
            .map( |d| d.len() as libc::size_t )
            .unwrap_or( 0 )
    }
}

pub struct RuntimeHandle {
    video_refresh_callback: libretro_sys::VideoRefreshFn,
    input_state_callback: libretro_sys::InputStateFn,
    audio_sample_batch_callback: libretro_sys::AudioSampleBatchFn,
    upload_video_frame_already_called: bool,
    audio_samples_uploaded: usize,

    video_width: u32,
    video_height: u32,
    video_frame_bytes_per_pixel: u32
}

impl RuntimeHandle {
    pub fn upload_video_frame( &mut self, data: &[u8] ) {
        assert!( self.upload_video_frame_already_called == false, "You can only call upload_video_frame() once per frame!" );
        assert!( data.len() as u32 >= self.video_width * self.video_height * self.video_frame_bytes_per_pixel, "Data too small to upload!" );

        self.upload_video_frame_already_called = true;
        let bytes = data.as_ptr() as *const libc::c_void;
        let width = self.video_width as libc::c_uint;
        let height = self.video_height as libc::c_uint;
        let bytes_per_line = (self.video_width * self.video_frame_bytes_per_pixel) as usize;
        unsafe {
            (self.video_refresh_callback)( bytes, width, height, bytes_per_line );
        }
    }

    pub fn upload_audio_frame( &mut self, data: &[i16] ) {
        assert!( data.len() % 2 == 0, "Audio data must be in stereo!" );

        self.audio_samples_uploaded += data.len();
        unsafe {
            (self.audio_sample_batch_callback)( data.as_ptr(), data.len() / 2 );
        }
    }

    pub fn is_joypad_button_pressed( &mut self, port: u32, button: JoypadButton ) -> bool {
        let device_id = match button {
            JoypadButton::A => libretro_sys::DEVICE_ID_JOYPAD_A,
            JoypadButton::B => libretro_sys::DEVICE_ID_JOYPAD_B,
            JoypadButton::X => libretro_sys::DEVICE_ID_JOYPAD_X,
            JoypadButton::Y => libretro_sys::DEVICE_ID_JOYPAD_Y,
            JoypadButton::Start => libretro_sys::DEVICE_ID_JOYPAD_START,
            JoypadButton::Select => libretro_sys::DEVICE_ID_JOYPAD_SELECT,
            JoypadButton::Left => libretro_sys::DEVICE_ID_JOYPAD_LEFT,
            JoypadButton::Right => libretro_sys::DEVICE_ID_JOYPAD_RIGHT,
            JoypadButton::Up => libretro_sys::DEVICE_ID_JOYPAD_UP,
            JoypadButton::Down => libretro_sys::DEVICE_ID_JOYPAD_DOWN,
            JoypadButton::L1 => libretro_sys::DEVICE_ID_JOYPAD_L,
            JoypadButton::L2 => libretro_sys::DEVICE_ID_JOYPAD_L2,
            JoypadButton::L3 => libretro_sys::DEVICE_ID_JOYPAD_L3,
            JoypadButton::R1 => libretro_sys::DEVICE_ID_JOYPAD_R,
            JoypadButton::R2 => libretro_sys::DEVICE_ID_JOYPAD_R2,
            JoypadButton::R3 => libretro_sys::DEVICE_ID_JOYPAD_R3
        };

        unsafe {
            let value = (self.input_state_callback)( port, libretro_sys::DEVICE_JOYPAD, 0, device_id );
            return value == 1;
        }
    }
}

#[doc(hidden)]
pub fn construct< T: 'static + Core >() -> Retro< T > {
    Retro::new( T::default() )
}

#[macro_export]
macro_rules! libretro_core {
    ($core: path) => (
        #[doc(hidden)]
        static mut LIBRETRO_INSTANCE: *mut $crate::Retro< $core > = 0 as *mut $crate::Retro< $core >;

        #[doc(hidden)]
        #[no_mangle]
        pub extern "C" fn retro_api_version() -> $crate::libc::c_uint {
            return $crate::libretro_sys::API_VERSION;
        }

        #[doc(hidden)]
        #[no_mangle]
        pub unsafe extern "C" fn retro_init() {
            assert_eq!( LIBRETRO_INSTANCE, 0 as *mut _ );
            let retro = $crate::construct::< $core >();
            LIBRETRO_INSTANCE = Box::into_raw( Box::new( retro ) );
        }

        #[doc(hidden)]
        #[no_mangle]
        pub unsafe extern "C" fn retro_deinit() {
            assert_ne!( LIBRETRO_INSTANCE, 0 as *mut _ );
            let instance = Box::from_raw( LIBRETRO_INSTANCE );
            LIBRETRO_INSTANCE = 0 as *mut _;
            ::std::mem::drop( instance );
        }

        #[doc(hidden)]
        #[no_mangle]
        pub unsafe extern "C" fn retro_set_environment( callback: $crate::libretro_sys::EnvironmentFn ) {
            $crate::Retro::< $core >::on_set_environment( callback )
        }

        #[doc(hidden)]
        #[no_mangle]
        pub unsafe extern "C" fn retro_set_video_refresh( callback: $crate::libretro_sys::VideoRefreshFn ) {
            assert_ne!( LIBRETRO_INSTANCE, 0 as *mut _ );
            (&mut *LIBRETRO_INSTANCE).on_set_video_refresh( callback )
        }

        #[doc(hidden)]
        #[no_mangle]
        pub unsafe extern "C" fn retro_set_audio_sample( callback: $crate::libretro_sys::AudioSampleFn ) {
            assert_ne!( LIBRETRO_INSTANCE, 0 as *mut _ );
            (&mut *LIBRETRO_INSTANCE).on_set_audio_sample( callback )
        }

        #[doc(hidden)]
        #[no_mangle]
        pub unsafe extern "C" fn retro_set_audio_sample_batch( callback: $crate::libretro_sys::AudioSampleBatchFn ) {
            assert_ne!( LIBRETRO_INSTANCE, 0 as *mut _ );
            (&mut *LIBRETRO_INSTANCE).on_set_audio_sample_batch( callback )
        }

        #[doc(hidden)]
        #[no_mangle]
        pub unsafe extern "C" fn retro_set_input_poll( callback: $crate::libretro_sys::InputPollFn ) {
            assert_ne!( LIBRETRO_INSTANCE, 0 as *mut _ );
            (&mut *LIBRETRO_INSTANCE).on_set_input_poll( callback )
        }

        #[doc(hidden)]
        #[no_mangle]
        pub unsafe extern "C" fn retro_set_input_state( callback: $crate::libretro_sys::InputStateFn ) {
            assert_ne!( LIBRETRO_INSTANCE, 0 as *mut _ );
            (&mut *LIBRETRO_INSTANCE).on_set_input_state( callback )
        }

        #[doc(hidden)]
        #[no_mangle]
        pub extern "C" fn retro_get_system_info( info: *mut $crate::libretro_sys::SystemInfo ) {
            $crate::Retro::< $core >::on_get_system_info( info )
        }

        #[doc(hidden)]
        #[no_mangle]
        pub unsafe extern "C" fn retro_get_system_av_info( info: *mut $crate::libretro_sys::SystemAvInfo ) {
            assert_ne!( LIBRETRO_INSTANCE, 0 as *mut _ );
            (&mut *LIBRETRO_INSTANCE).on_get_system_av_info( info )
        }

        #[doc(hidden)]
        #[no_mangle]
        pub unsafe extern "C" fn retro_set_controller_port_device( port: $crate::libc::c_uint, device: $crate::libc::c_uint ) {
            assert_ne!( LIBRETRO_INSTANCE, 0 as *mut _ );
            (&mut *LIBRETRO_INSTANCE).on_set_controller_port_device( port, device )
        }

        #[doc(hidden)]
        #[no_mangle]
        pub unsafe extern "C" fn retro_reset() {
            assert_ne!( LIBRETRO_INSTANCE, 0 as *mut _ );
            (&mut *LIBRETRO_INSTANCE).on_reset()
        }

        #[doc(hidden)]
        #[no_mangle]
        pub unsafe extern "C" fn retro_run() {
            assert_ne!( LIBRETRO_INSTANCE, 0 as *mut _ );
            (&mut *LIBRETRO_INSTANCE).on_run()
        }

        #[doc(hidden)]
        #[no_mangle]
        pub unsafe extern "C" fn retro_serialize_size() -> $crate::libc::size_t {
            assert_ne!( LIBRETRO_INSTANCE, 0 as *mut _ );
            (&mut *LIBRETRO_INSTANCE).on_serialize_size()
        }

        #[doc(hidden)]
        #[no_mangle]
        pub unsafe extern "C" fn retro_serialize( data: *mut $crate::libc::c_void, size: $crate::libc::size_t ) -> bool {
            assert_ne!( LIBRETRO_INSTANCE, 0 as *mut _ );
            (&mut *LIBRETRO_INSTANCE).on_serialize( data, size )
        }

        #[doc(hidden)]
        #[no_mangle]
        pub unsafe extern "C" fn retro_unserialize( data: *const $crate::libc::c_void, size: $crate::libc::size_t ) -> bool {
            assert_ne!( LIBRETRO_INSTANCE, 0 as *mut _ );
            (&mut *LIBRETRO_INSTANCE).on_unserialize( data, size )
        }

        #[doc(hidden)]
        #[no_mangle]
        pub unsafe extern "C" fn retro_cheat_reset() {
            assert_ne!( LIBRETRO_INSTANCE, 0 as *mut _ );
            (&mut *LIBRETRO_INSTANCE).on_cheat_reset()
        }

        #[doc(hidden)]
        #[no_mangle]
        pub unsafe extern "C" fn retro_cheat_set( index: $crate::libc::c_uint, is_enabled: bool, code: *const $crate::libc::c_char ) {
            assert_ne!( LIBRETRO_INSTANCE, 0 as *mut _ );
            (&mut *LIBRETRO_INSTANCE).on_cheat_set( index, is_enabled, code )
        }

        #[doc(hidden)]
        #[no_mangle]
        pub unsafe extern "C" fn retro_load_game( game: *const $crate::libretro_sys::GameInfo ) -> bool {
            assert_ne!( LIBRETRO_INSTANCE, 0 as *mut _ );
            (&mut *LIBRETRO_INSTANCE).on_load_game( game )
        }

        #[doc(hidden)]
        #[no_mangle]
        pub unsafe extern "C" fn retro_load_game_special( game_type: $crate::libc::c_uint, info: *const $crate::libretro_sys::GameInfo, num_info: $crate::libc::size_t ) -> bool {
            assert_ne!( LIBRETRO_INSTANCE, 0 as *mut _ );
            (&mut *LIBRETRO_INSTANCE).on_load_game_special( game_type, info, num_info )
        }

        #[doc(hidden)]
        #[no_mangle]
        pub unsafe extern "C" fn retro_unload_game() {
            assert_ne!( LIBRETRO_INSTANCE, 0 as *mut _ );
            (&mut *LIBRETRO_INSTANCE).on_unload_game()
        }

        #[doc(hidden)]
        #[no_mangle]
        pub unsafe extern "C" fn retro_get_region() -> $crate::libc::c_uint {
            assert_ne!( LIBRETRO_INSTANCE, 0 as *mut _ );
            (&mut *LIBRETRO_INSTANCE).on_get_region()
        }

        #[doc(hidden)]
        #[no_mangle]
        pub unsafe extern "C" fn retro_get_memory_data( id: $crate::libc::c_uint ) -> *mut $crate::libc::c_void {
            assert_ne!( LIBRETRO_INSTANCE, 0 as *mut _ );
            (&mut *LIBRETRO_INSTANCE).on_get_memory_data( id )
        }

        #[doc(hidden)]
        #[no_mangle]
        pub unsafe extern "C" fn retro_get_memory_size( id: $crate::libc::c_uint ) -> $crate::libc::size_t {
            assert_ne!( LIBRETRO_INSTANCE, 0 as *mut _ );
            (&mut *LIBRETRO_INSTANCE).on_get_memory_size( id )
        }
    )
}
