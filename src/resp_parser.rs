use std::convert::TryInto;

#[derive(Debug, PartialEq)]
pub enum DataType {
    SimpleString(String),
    Error(String),
    Integer(i64),
    BulkString(i64, String),
    Array(i64, Vec<DataType>),
}

pub struct RespParser<'parser> {
    pub index: usize,
    pub commands: Vec<&'parser str>,
}

impl<'parser> RespParser<'parser> {
    pub fn new(slice: &'parser [u8]) -> Self {
        let string = std::str::from_utf8(slice).unwrap();
        let commands: Vec<&str> = string.split("\r\n").collect();

        Self { index: 0, commands }
    }

    pub fn parse(&mut self) -> Vec<DataType> {
        self.m_parse(None)
    }

    fn m_parse(&mut self, count: Option<i64>) -> Vec<DataType> {
        let mut ret = Vec::new();
        while self.index < self.commands.len() - 1 {
            let command = self.commands.get(self.index).unwrap();
            let first_byte = &command.as_bytes()[..1];
            let data_type = match first_byte {
                b"+" => self.parse_simple_string(),
                b"-" => self.parse_error(),
                b":" => self.parse_integer(),
                b"$" => self.parse_bulk_string(),
                b"*" => self.parse_array(),
                _ => todo!(),
            };
            ret.push(data_type);

            if let Some(value) = count {
                if ret.len() >= value.try_into().unwrap() {
                    break;
                }
            }
        }

        ret
    }

    fn parse_simple_string(&mut self) -> DataType {
        let command = self.commands.get(self.index).unwrap();
        let data = &command[1..];
        self.index += 1;

        DataType::SimpleString(data.to_owned())
    }

    fn parse_integer(&mut self) -> DataType {
        let command = self.commands.get(self.index).unwrap();
        let data = &command[1..];
        self.index += 1;

        DataType::Integer(data.parse::<i64>().unwrap())
    }

    fn parse_error(&mut self) -> DataType {
        let command = self.commands.get(self.index).unwrap();
        let data = &command[1..];
        self.index += 1;

        DataType::Error(data.to_owned())
    }

    fn parse_bulk_string(&mut self) -> DataType {
        let command = self.commands.get(self.index).unwrap();
        let count = (&command[1..]).parse::<i64>().unwrap();
        self.index += 1;

        let command = self.commands.get(self.index).unwrap();
        let value = (&command).to_string();
        self.index += 1;

        DataType::BulkString(count, value)
    }

    fn parse_array(&mut self) -> DataType {
        let command = self.commands.get(self.index).unwrap();
        let count = (&command[1..]).parse::<i64>().unwrap();
        self.index += 1;
        let res = self.m_parse(Some(count));

        DataType::Array(count, res)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parser_simple_string_test() {
        let mut parser = RespParser::new(b"+simple\r\n+string\r\n");
        let res = parser.parse();

        assert_eq!(
            res,
            vec![
                DataType::SimpleString("simple".to_owned()),
                DataType::SimpleString("string".to_owned()),
            ]
        );
    }

    #[test]
    fn parser_bulk_string_test() {
        let mut parser = RespParser::new(b"$5\r\nhello\r\n$11\r\nhello world\r\n");
        let res = parser.parse();

        assert_eq!(
            res,
            vec![
                DataType::BulkString(5, "hello".to_owned()),
                DataType::BulkString(11, "hello world".to_owned()),
            ]
        );
    }

    #[test]
    fn parser_error_test() {
        let mut parser = RespParser::new(b"-ERR unknown command \"helloworld\"\r\n");
        let res = parser.parse();

        assert_eq!(
            res,
            vec![DataType::Error(
                "ERR unknown command \"helloworld\"".to_owned()
            ),]
        );
    }

    #[test]
    fn parser_integer_test() {
        let mut parser = RespParser::new(b":0\r\n:1\r\n:2\r\n:3\r\n:4\r\n:5\r\n:100\r\n");
        let res = parser.parse();

        assert_eq!(
            res,
            vec![
                DataType::Integer(0),
                DataType::Integer(1),
                DataType::Integer(2),
                DataType::Integer(3),
                DataType::Integer(4),
                DataType::Integer(5),
                DataType::Integer(100),
            ]
        );
    }

    #[test]
    fn parser_array_test() {
        let mut parser = RespParser::new(b"*5\r\n$1\r\ntest\r\n:2\r\n:3\r\n:4\r\n$5\r\nhello\r\n");
        let res = parser.parse();

        assert_eq!(
            res,
            vec![DataType::Array(
                5,
                vec![
                    DataType::BulkString(1, "test".to_owned()),
                    DataType::Integer(2),
                    DataType::Integer(3),
                    DataType::Integer(4),
                    DataType::BulkString(5, "hello".to_owned()),
                ]
            )]
        );
    }

    #[test]
    fn parser_nested_array_test() {
        let mut parser = RespParser::new(b"*2\r\n*2\r\n:1\r\n:2\r\n*2\r\n$5\r\nhello\r\n:3\r\n");
        let res = parser.parse();

        assert_eq!(
            res,
            vec![DataType::Array(
                2,
                vec![
                    DataType::Array(2, vec![DataType::Integer(1), DataType::Integer(2)]),
                    DataType::Array(
                        2,
                        vec![
                            DataType::BulkString(5, "hello".to_owned()),
                            DataType::Integer(3)
                        ]
                    )
                ]
            )]
        )
    }
}
