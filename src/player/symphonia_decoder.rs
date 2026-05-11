use std::io::{self, Read, Seek, SeekFrom};
use log::{debug, error};
use symphonia::core::audio::SampleBuffer;
use symphonia::core::codecs::DecoderOptions;
use symphonia::core::formats::FormatOptions;
use symphonia::core::io::{MediaSource, MediaSourceStream};
use symphonia::core::meta::MetadataOptions;
use symphonia::core::probe::Hint;

pub struct AudioInfo {
    pub sample_rate: u32,
    pub channels: u16,
}

/// Wraps any `Read` into a non-seekable `MediaSource` for symphonia.
struct ReadOnlySource<R: Read + Send + Sync> {
    inner: R,
    position: u64,
}

impl<R: Read + Send + Sync> ReadOnlySource<R> {
    fn new(reader: R) -> Self {
        Self { inner: reader, position: 0 }
    }
}

impl<R: Read + Send + Sync> Read for ReadOnlySource<R> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let n = self.inner.read(buf)?;
        self.position += n as u64;
        Ok(n)
    }
}

impl<R: Read + Send + Sync> Seek for ReadOnlySource<R> {
    fn seek(&mut self, _pos: SeekFrom) -> io::Result<u64> {
        Err(io::Error::new(io::ErrorKind::Unsupported, "stream is not seekable"))
    }
}

impl<R: Read + Send + Sync + 'static> MediaSource for ReadOnlySource<R> {
    fn is_seekable(&self) -> bool {
        false
    }
    fn byte_len(&self) -> Option<u64> {
        None
    }
}

pub fn decode_streaming<R, F>(reader: R, mut on_packet: F) -> Result<AudioInfo, String>
where
    R: Read + Send + Sync + 'static,
    F: FnMut(&[i16]) -> bool,
{
    let source = ReadOnlySource::new(reader);
    let mss = MediaSourceStream::new(Box::new(source), Default::default());

    let hint = Hint::new();
    let format_opts = FormatOptions::default();
    let metadata_opts = MetadataOptions::default();
    let decoder_opts = DecoderOptions::default();

    let probed = symphonia::default::get_probe()
        .format(&hint, mss, &format_opts, &metadata_opts)
        .map_err(|e| format!("Probe error: {:?}", e))?;

    let mut format = probed.format;

    let track = format.default_track()
        .ok_or("No default track found")?;

    let sample_rate = track.codec_params.sample_rate.unwrap_or(44100);
    let channels = track.codec_params.channels.map(|c| c.count() as u16).unwrap_or(2);
    let track_id = track.id;

    debug!("[SymphoniaDecoder] sample_rate: {}, channels: {}", sample_rate, channels);

    let mut decoder = symphonia::default::get_codecs()
        .make(&track.codec_params, &decoder_opts)
        .map_err(|e| format!("Codec error: {:?}", e))?;

    // Allocate SampleBuffer once and reuse across packets to avoid per-packet allocations
    let mut sample_buf: Option<SampleBuffer<i16>> = None;

    loop {
        let packet = match format.next_packet() {
            Ok(packet) => packet,
            Err(symphonia::core::errors::Error::IoError(ref e))
                if e.kind() == std::io::ErrorKind::UnexpectedEof => break,
            Err(e) => {
                error!("[SymphoniaDecoder] Packet read error: {:?}", e);
                break;
            }
        };

        if packet.track_id() != track_id {
            continue;
        }

        let decoded = match decoder.decode(&packet) {
            Ok(decoded) => decoded,
            Err(e) => {
                error!("[SymphoniaDecoder] Decode error: {:?}", e);
                continue;
            }
        };

        let spec = *decoded.spec();
        let num_frames = decoded.frames();
        let needed = num_frames * spec.channels.count();

        // Reuse buffer if capacity is sufficient, otherwise reallocate
        let buf = match sample_buf.as_mut() {
            Some(b) if b.capacity() >= needed => b,
            _ => {
                sample_buf = Some(SampleBuffer::<i16>::new(num_frames as u64, spec));
                sample_buf.as_mut().unwrap()
            }
        };
        buf.copy_interleaved_ref(decoded);

        if !on_packet(buf.samples()) {
            debug!("[SymphoniaDecoder] Streaming stopped by callback");
            break;
        }
    }

    Ok(AudioInfo { sample_rate, channels })
}
