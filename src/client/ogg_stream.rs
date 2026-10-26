//! Music streamed to kira from an Ogg Vorbis file, in place of kira's own streaming decoder.
//!
//! kira takes a track's length from the file's header and, before a loop starts over,
//! waits for every frame of it. symphonia 0.6 decodes many Vorbis files (the original's
//! too) a few thousand frames short of that length, so kira waited for ever and the music
//! fell silent instead of looping. Here the missing frames are silence, and going back
//! reopens the file, because symphonia also refuses to seek some files back to their start.

use kira::sound::streaming::Decoder;
use kira::sound::FromFileError;
use kira::Frame;
use std::io::Cursor;
use std::sync::Arc;
use symphonia::core::codecs::audio::AudioDecoder;
use symphonia::core::codecs::CodecParameters;
use symphonia::core::formats::probe::Hint;
use symphonia::core::formats::{FormatReader, TrackType};
use symphonia::core::io::MediaSourceStream;

/// Frames of silence handed over at a time past the decoded end.
const SILENCE_CHUNK: usize = 1024;

pub struct OggStream {
    bytes: Arc<[u8]>,
    reader: Box<dyn FormatReader>,
    decoder: Box<dyn AudioDecoder>,
    track_id: u32,
    sample_rate: u32,
    /// The length the header claims, which kira plays to the end.
    num_frames: usize,
    /// Frames handed to kira since the start, decoded or silence.
    position: usize,
    samples: Vec<f32>,
}

impl OggStream {
    pub fn open(bytes: Arc<[u8]>) -> Result<OggStream, FromFileError> {
        let source = MediaSourceStream::new(Box::new(Cursor::new(bytes.clone())), Default::default());
        let reader = symphonia::default::get_probe().probe(&Hint::default(), source, Default::default(), Default::default())?;
        let track = reader.default_track(TrackType::Audio).ok_or(FromFileError::NoDefaultTrack)?;
        let Some(CodecParameters::Audio(params)) = &track.codec_params else { return Err(FromFileError::NoDefaultTrack) };
        let sample_rate = params.sample_rate.ok_or(FromFileError::UnknownSampleRate)?;
        let num_frames = track.num_frames.ok_or(FromFileError::UnknownDuration)? as usize;
        let decoder = symphonia::default::get_codecs().make_audio_decoder(params, &Default::default())?;
        let track_id = track.id;
        Ok(OggStream { bytes, reader, decoder, track_id, sample_rate, num_frames, position: 0, samples: Vec::new() })
    }

    fn silence_to_the_end(&mut self) -> Vec<Frame> {
        let missing = self.num_frames.saturating_sub(self.position).min(SILENCE_CHUNK);
        self.position += missing;
        vec![Frame::ZERO; missing]
    }
}

impl Decoder for OggStream {
    type Error = FromFileError;

    fn sample_rate(&self) -> u32 {
        self.sample_rate
    }

    fn num_frames(&self) -> usize {
        self.num_frames
    }

    fn decode(&mut self) -> Result<Vec<Frame>, FromFileError> {
        loop {
            let Some(packet) = self.reader.next_packet()? else { return Ok(self.silence_to_the_end()) };
            if packet.track_id != self.track_id {
                continue;
            }
            let buffer = self.decoder.decode(&packet)?;
            buffer.copy_to_vec_interleaved(&mut self.samples);
            let frames: Vec<Frame> = match buffer.num_planes() {
                1 => self.samples.iter().map(|&sample| Frame::from_mono(sample)).collect(),
                2 => self.samples.chunks_exact(2).map(|pair| Frame::new(pair[0], pair[1])).collect(),
                _ => return Err(FromFileError::UnsupportedChannelConfiguration),
            };
            self.position += frames.len();
            return Ok(frames);
        }
    }

    /// Always back to the start, which the trait allows ("an earlier sample than the
    /// one requested"): kira then decodes forward to `index`. Music only seeks to loop.
    fn seek(&mut self, _index: usize) -> Result<usize, FromFileError> {
        *self = OggStream::open(self.bytes.clone())?;
        Ok(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// menu.ogg decodes 1792 frames short of its header: the stream still hands over
    /// exactly the header's length, then starts over from the first frame.
    #[test]
    fn a_short_decoding_track_plays_its_whole_length_and_starts_over() {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("Resources/Sounds/Music/menu.ogg");
        let mut stream = OggStream::open(std::fs::read(path).unwrap().into()).unwrap();
        // A Vorbis stream's first packet decodes to nothing, so empty chunks are allowed,
        // but not the endless run of them kira's own decoder gave at the end.
        let first_frames = |stream: &mut OggStream| (0..4).map(|_| stream.decode().unwrap()).find(|frames| !frames.is_empty()).unwrap();
        let first = first_frames(&mut stream);
        let mut handed = first.len();
        let mut empty_in_a_row = 0;
        while handed < stream.num_frames() {
            let frames = stream.decode().unwrap();
            empty_in_a_row = if frames.is_empty() { empty_in_a_row + 1 } else { 0 };
            assert!(empty_in_a_row < 4, "stuck at frame {handed} of {}", stream.num_frames());
            handed += frames.len();
        }
        assert_eq!(handed, stream.num_frames());
        assert_eq!(stream.seek(0).unwrap(), 0);
        assert_eq!(first_frames(&mut stream), first, "the same first frames again");
    }
}
