use quick_protobuf::sizeofs::{sizeof_len, sizeof_varint};
use quick_protobuf::{MessageWrite, Writer, WriterBackend};

use super::proto::{Pair, Log, LogGroup, LogGroupList};

impl<'a> MessageWrite for Log<'a> {
    fn write_message<W: WriterBackend>(&self, w: &mut Writer<W>) -> quick_protobuf::Result<()> {
        w.write_with_tag(8, |w| w.write_uint32(self.time))?;
        for s in &self.contents {
            w.write_with_tag(18, |w| w.write_message(s))?;
        }
        Ok(())
    }

    fn get_size(&self) -> usize {
        1 + sizeof_varint(self.time as u64)
            + self
            .contents
            .iter()
            .map(|s| 1 + sizeof_len((s).get_size()))
            .sum::<usize>()
    }
}

impl<'a> MessageWrite for Pair<'a> {
    fn write_message<W: WriterBackend>(&self, w: &mut Writer<W>) -> quick_protobuf::Result<()> {
        w.write_with_tag(10, |w| w.write_string(&*self.key))?;
        w.write_with_tag(18, |w| w.write_string(&*self.value))?;
        Ok(())
    }

    fn get_size(&self) -> usize {
        2 + sizeof_len((&self.key).len()) + sizeof_len((&self.value).len())
    }
}

impl<'a> MessageWrite for LogGroup<'a> {
    fn write_message<W: WriterBackend>(&self, w: &mut Writer<W>) -> quick_protobuf::Result<()> {
        for s in &self.logs {
            w.write_with_tag(10, |w| w.write_message(s))?;
        }
        // if let Some(ref s) = self.reserved { w.write_with_tag(18, |w| w.write_string(&**s))?; }
        if let Some(ref s) = self.topic {
            w.write_with_tag(26, |w| w.write_string(&**s))?;
        }
        if let Some(ref s) = self.source {
            w.write_with_tag(34, |w| w.write_string(&**s))?;
        }
        for s in &self.log_tags {
            w.write_with_tag(50, |w| w.write_message(s))?;
        }
        Ok(())
    }

    fn get_size(&self) -> usize {
        self.logs
            .iter()
            .map(|s| 1 + sizeof_len((s).get_size()))
            .sum::<usize>()
            // + self
            //     .reserved
            //     .as_ref()
            //     .map_or(0, |m| 1 + sizeof_len((m).len()))
            + self.topic.as_ref().map_or(0, |m| 1 + sizeof_len((m).len()))
            + self
            .source
            .as_ref()
            .map_or(0, |m| 1 + sizeof_len((m).len()))
            + self
            .log_tags
            .iter()
            .map(|s| 1 + sizeof_len((s).get_size()))
            .sum::<usize>()
    }
}

impl<'a> MessageWrite for LogGroupList<'a> {
    fn write_message<W: WriterBackend>(&self, w: &mut Writer<W>) -> quick_protobuf::Result<()> {
        for s in &self.log_group_list {
            w.write_with_tag(10, |w| w.write_message(s))?;
        }
        Ok(())
    }

    fn get_size(&self) -> usize {
        self.log_group_list
            .iter()
            .map(|s| 1 + sizeof_len((s).get_size()))
            .sum::<usize>()
    }
}

