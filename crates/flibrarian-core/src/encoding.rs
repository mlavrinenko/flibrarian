use chardetng::EncodingDetector;
use encoding_rs_io::{DecodeReaderBytes, DecodeReaderBytesBuilder};
use std::io::{self, BufRead, BufReader, Cursor, Read};

type NonUtf8BufReader<R> = BufReader<DecodeReaderBytes<io::Chain<Cursor<Vec<u8>>, R>, Vec<u8>>>;

pub enum Utf8Reader<R> {
    Utf8(io::Chain<Cursor<Vec<u8>>, R>),
    NonUtf8(NonUtf8BufReader<R>),
}

impl<R: Read> Read for Utf8Reader<R> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        match self {
            Self::Utf8(chain) => chain.read(buf),
            Self::NonUtf8(reader) => reader.read(buf),
        }
    }
}

impl<R: BufRead> BufRead for Utf8Reader<R> {
    fn fill_buf(&mut self) -> io::Result<&[u8]> {
        match self {
            Self::Utf8(chain) => chain.fill_buf(),
            Self::NonUtf8(reader) => reader.fill_buf(),
        }
    }

    fn consume(&mut self, amt: usize) {
        match self {
            Self::Utf8(chain) => chain.consume(amt),
            Self::NonUtf8(reader) => reader.consume(amt),
        }
    }
}

const SAMPLE_SIZE: usize = 4096;

pub fn to_utf8_reader<R: BufRead>(mut reader: R) -> io::Result<Utf8Reader<R>> {
    let mut sample = Vec::with_capacity(SAMPLE_SIZE);
    while sample.len() < SAMPLE_SIZE {
        let buf = reader.fill_buf()?;
        if buf.is_empty() {
            break;
        }
        let to_read = (SAMPLE_SIZE - sample.len()).min(buf.len());
        sample.extend_from_slice(&buf[..to_read]);
        reader.consume(to_read);
    }

    // Handle BOMs
    if sample.starts_with(&[0xEF, 0xBB, 0xBF]) {
        // UTF-8 BOM
        sample.drain(..3);
        let cursor = Cursor::new(sample);
        return Ok(Utf8Reader::Utf8(cursor.chain(reader)));
    } else if sample.starts_with(&[0xFF, 0xFE]) {
        // UTF-16 LE BOM
        sample.drain(..2);
        return create_non_utf8_reader(sample, reader, encoding_rs::UTF_16LE, false, false);
    } else if sample.starts_with(&[0xFE, 0xFF]) {
        // UTF-16 BE BOM
        sample.drain(..2);
        return create_non_utf8_reader(sample, reader, encoding_rs::UTF_16BE, false, false);
    }

    // Check if it's valid UTF-8 without BOM
    if std::str::from_utf8(&sample).is_ok() {
        let cursor = Cursor::new(sample);
        return Ok(Utf8Reader::Utf8(cursor.chain(reader)));
    }

    // Use chardetng for other encodings
    let mut detector = EncodingDetector::new();
    detector.feed(&sample, false);
    let encoding = detector.guess(None, true);

    create_non_utf8_reader(sample, reader, encoding, true, true)
}

fn create_non_utf8_reader<R: BufRead>(
    sample: Vec<u8>,
    reader: R,
    encoding: &'static encoding_rs::Encoding,
    bom_sniffing: bool,
    strip_bom: bool,
) -> io::Result<Utf8Reader<R>> {
    let sample_reader = Cursor::new(sample);
    let full_reader = sample_reader.chain(reader);

    let mut builder = DecodeReaderBytesBuilder::new();
    builder
        .encoding(Some(encoding))
        .bom_sniffing(bom_sniffing)
        .strip_bom(strip_bom);

    let buffer_size = std::cmp::max(SAMPLE_SIZE, 4);
    let decode_reader = builder.build_with_buffer(full_reader, vec![0; buffer_size])?;

    let buffered_reader = BufReader::new(decode_reader);
    Ok(Utf8Reader::NonUtf8(buffered_reader))
}

pub fn bytes_to_utf8_string(raw_bytes: &[u8]) -> io::Result<String> {
    let byte_reader = BufReader::new(raw_bytes);
    let mut reader = to_utf8_reader(byte_reader)?;

    let mut content = String::new();
    let mut buffer = [0; 8192];

    loop {
        match reader.read(&mut buffer) {
            Ok(0) => break,
            Ok(n) => content.push_str(&String::from_utf8_lossy(&buffer[..n])),
            Err(e) => return Err(e),
        }
    }

    Ok(content)
}
