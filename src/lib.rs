use core::str;
use std::str::FromStr;

const SEQUENCE_FIRST_CHECKSUM_DIGITS: &'static [u8; 9] = &[3, 7, 6, 1, 8, 9, 4, 5, 2];
const SEQUENCE_SECOND_CHECKSUM_DIGITS: &'static [u8; 10] = &[5, 4, 3, 2, 7, 6, 5, 4, 3, 2];
const TIN_LENGTH: usize = 11;
const ORG_LENGTH: usize = 9;
const SEQUENCE_ORG_CHECKSUM_DIGITS: &'static [u8; 8] = &[3, 2, 7, 6, 5, 4, 3, 2];

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
#[non_exhaustive]
pub enum PersonKind {
    Normal,
    HNumber,
    Anonymous,
    Synthetic,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct PersonNumber {
    kind: PersonKind,
    value: [u8; TIN_LENGTH],
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct OrgNumber {
    value: [u8; ORG_LENGTH],
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
#[non_exhaustive]
pub enum NorwegianTin {
    FNumber(PersonNumber),
    DNumber(PersonNumber),
    OrgNumber(OrgNumber),
}

#[derive(Debug, PartialEq)]
pub enum NorwegianTinError {
    InvalidLength,
    NonNumericValue,
    InvalidChecksum,
    InvalidDate,
}

impl std::fmt::Display for NorwegianTinError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NorwegianTinError::InvalidLength => write!(f, "InvalidLength"),
            NorwegianTinError::NonNumericValue => write!(f, "NonNumericValue"),
            NorwegianTinError::InvalidChecksum => write!(f, "InvalidChecksum"),
            NorwegianTinError::InvalidDate => write!(f, "InvalidDate"),
        }
    }
}

impl FromStr for NorwegianTin {
    type Err = NorwegianTinError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::parse(s)
    }
}

impl Into<String> for NorwegianTin {
    fn into(self) -> String {
        let bytes = self.get_value();
        let s: String = bytes.iter().map(|&d| (d + b'0') as char).collect();
        s
    }
}

impl AsRef<[u8]> for NorwegianTin {
    fn as_ref(&self) -> &[u8] {
        self.get_value()
    }
}

impl std::fmt::Display for NorwegianTin {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s: String = self.clone().into();
        let kind = match self.get_kind() {
            PersonKind::Anonymous => " (Anonymous) ",
            PersonKind::HNumber => " (H-Number) ",
            PersonKind::Synthetic => " (Synthetic) ",
            PersonKind::Normal => "",
        };
        // Masking the last 5 digits for privacy
        write!(f, "{}{}*****", kind, &s[0..6])
    }
}
impl NorwegianTin {
    pub fn get_value(&self) -> &[u8] {
        match self {
            NorwegianTin::FNumber(fnr) => &fnr.value,
            NorwegianTin::DNumber(dnr) => &dnr.value,
            NorwegianTin::OrgNumber(org) => &org.value,
        }
    }
    pub fn get_kind(&self) -> PersonKind {
        match self {
            NorwegianTin::FNumber(fnr) => fnr.kind,
            NorwegianTin::DNumber(dnr) => dnr.kind,
            NorwegianTin::OrgNumber(_) => PersonKind::Normal, // Org numbers are not categorized by kind
        }
    }

    pub fn parse(s: &str) -> Result<NorwegianTin, NorwegianTinError> {
        let bytes = s.as_bytes();
        if bytes.len() != TIN_LENGTH && bytes.len() != ORG_LENGTH {
            return Err(NorwegianTinError::InvalidLength);
        }

        let mut digits = [0u8; TIN_LENGTH];
        for (i, &b) in bytes.iter().enumerate() {
            if b < b'0' || b > b'9' {
                return Err(NorwegianTinError::NonNumericValue);
            }
            digits[i] = b - b'0';
        }
        if bytes.len() == ORG_LENGTH {
            let _ =
                Self::calculate_checksum(
                    &digits[0..8],
                    SEQUENCE_ORG_CHECKSUM_DIGITS,
                    |r| match (11 - r) % 11 {
                        10 => Err(NorwegianTinError::InvalidChecksum),
                        v if v == digits[8] => Ok(()),
                        _ => Err(NorwegianTinError::InvalidChecksum),
                    },
                )?;

            return Ok(NorwegianTin::OrgNumber(OrgNumber {
                value: digits[0..9].try_into().unwrap(),
            }));
        }

        let _ =
            Self::calculate_checksum(&digits[0..9], SEQUENCE_FIRST_CHECKSUM_DIGITS, |r| {
                match (r + digits[9]) % 11 {
                    0..=3 => Ok(()),
                    _ => Err(NorwegianTinError::InvalidChecksum),
                }
            })?;
        let _ =
            Self::calculate_checksum(
                &digits[0..10],
                SEQUENCE_SECOND_CHECKSUM_DIGITS,
                |r| match (r + digits[10]) % 11 {
                    0 => Ok(()),
                    _ => Err(NorwegianTinError::InvalidChecksum),
                },
            )?;
        let kind = Self::check_kind(digits[2])?;

        let day = digits[0] * 10 + digits[1];
        let month = kind.get_base_month(digits[2] * 10 + digits[3]);
        let year = digits[4] as u16 * 10 + digits[5] as u16;
        // Determine if it's a D-number or F-number
        match digits[0] {
            0..=3 => {
                // F-number
                if !Self::is_valid_date(day, month, year) {
                    return Err(NorwegianTinError::InvalidDate);
                }
                Ok(NorwegianTin::FNumber(PersonNumber {
                    kind: kind,
                    value: digits,
                }))
            }
            4..=7 => {
                let actual_day = day - 40;
                if !Self::is_valid_date(actual_day, month, year) {
                    return Err(NorwegianTinError::InvalidDate);
                }
                Ok(NorwegianTin::DNumber(PersonNumber {
                    kind: kind,
                    value: digits,
                }))
            }
            _ => Err(NorwegianTinError::InvalidDate),
        }
    }

    fn check_kind(month: u8) -> Result<PersonKind, NorwegianTinError> {
        match month {
            0..=1 => Ok(PersonKind::Normal),
            4..=5 => Ok(PersonKind::HNumber),
            6..=7 => Ok(PersonKind::Anonymous),
            8..=9 => Ok(PersonKind::Synthetic),
            _ => Err(NorwegianTinError::InvalidDate),
        }
    }

    fn calculate_checksum<T: FnOnce(u8) -> Result<(), NorwegianTinError>>(
        digits: &[u8],
        weights: &[u8],
        matcher: T,
    ) -> Result<(), NorwegianTinError> {
        let sum: u32 = weights
            .iter()
            .zip(digits.iter())
            .map(|(&w, &d)| w as u32 * d as u32)
            .sum();
        let remainder = (sum % 11) as u8;
        matcher(remainder)
    }

    fn is_valid_date(day: u8, month: u8, year: u16) -> bool {
        if month == 0 || month > 12 || day == 0 || year >= 100 {
            return false;
        }
        let days_in_month = match month {
            1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
            4 | 6 | 9 | 11 => 30,
            2 => {
                if year % 4 == 0 {
                    29
                } else {
                    28
                }
            }
            _ => return false,
        };
        day <= days_in_month
    }
}

impl PersonKind {
    pub fn is_test_id(&self) -> bool {
        match self {
            PersonKind::Normal => false,
            PersonKind::HNumber => true,
            PersonKind::Anonymous => true,
            PersonKind::Synthetic => true,
        }
    }
    fn get_base_month(&self, month: u8) -> u8 {
        match self {
            PersonKind::Normal => month,
            PersonKind::HNumber => month - 40,
            PersonKind::Anonymous => month - 60,
            PersonKind::Synthetic => month - 80,
        }
    }
}

#[cfg(test)]
mod test {
    use std::vec;

    pub use super::*;
    #[test]
    fn test_invalid_length() {
        let tins = vec!["0123456789", "123456789012", "123", "12345678",""];
        for tin in tins {
            assert_eq!(
                NorwegianTin::parse(tin).unwrap_err(),
                NorwegianTinError::InvalidLength
            );
        }
    }
    #[test]
    fn test_invalid_characters() {
        let tins = vec!["1234567890a", "abcdefghijk", "12345abc678"];
        for tin in tins {
            assert_eq!(
                NorwegianTin::parse(tin).unwrap_err(),
                NorwegianTinError::NonNumericValue
            );
        }
    }
    #[test]
    fn test_invalid_checksum() {
        let tins = vec![
            "12345678901",
            "11111111111",
            "22222222222",
            "33333333333",
            "44444444444",
            "55555555555",
            "66666666666",
            "77777777777",
            "88888888888",
            "99999999999",
            "12345678912",
            "98765432109",
        ];
        for tin in tins {
            assert_eq!(
                NorwegianTin::parse(tin).unwrap_err(),
                NorwegianTinError::InvalidChecksum
            );
        }
    }

    #[test]
    fn test_2032_format() {
        let tins = vec!["11010000000", "11010000019", "11010000027", "11010000035"];
        for tin in tins {
            assert!(NorwegianTin::parse(tin).is_ok());
        }
    }

    #[test]
    fn test_invalid_date() {
        let tins = vec!["00000000000", "11001000073"];
        for tin in tins {
            assert_eq!(
                NorwegianTin::parse(tin).unwrap_err(),
                NorwegianTinError::InvalidDate
            );
        }
    }

    #[test]
    fn test_display() {
        let err: NorwegianTinError = "00000000000".parse::<NorwegianTin>().unwrap_err();
        assert_eq!(format!("{}", err), "InvalidDate");

        let tin: NorwegianTin = "16057902284".parse().unwrap();
        assert_eq!(format!("{}", tin), "160579*****");

        let tin: NorwegianTin = "22517149261".parse().unwrap();
        assert_eq!(format!("{}", tin), " (H-Number) 225171*****");

        let tin: NorwegianTin = "08639815316".parse().unwrap();
        assert_eq!(format!("{}", tin), " (Anonymous) 086398*****");

        let tin: NorwegianTin = "70887100797".parse().unwrap();
        assert_eq!(format!("{}", tin), " (Synthetic) 708871*****");
    }

    #[test]
    fn test_valid_f_number() {
        let tins = vec![
            "16057902284",
            "09063332523",
            "17068632781",
            "20069290990",
            "06071732280",
            "14033147466",
            "24059638544",
            "26073600501",
            "24043444170",
            "06113401662",
            "25098514123",
            "04015896960",
            "28076429032",
            "27017709351",
            "17051023862",
            "10093039144",
            "16048999194",
            "15019401615",
            "08057294664",
            "19045132024",
            "29067796398",
            "14056549677",
            "26111739128",
            "10103621777",
            "17120108360",
            "24041646671",
            "23082311440",
            "12042637996",
            "01052609131",
            "10016402791",
            "10023737472",
            "05048334972",
            "23129407840",
            "20090624106",
            "17047438926",
            "28093304286",
            "24052024052",
            "28128590643",
            "12123717110",
            "30123017591",
            "17093748212",
            "24084932503",
            "30091723170",
            "16032039514",
            "09100106539",
            "03078922159",
            "23016141046",
            "07119746641",
            "14093744021",
            "19057647348",
            "02013299997",
        ];
        for tin in tins {
            assert_eq!(NorwegianTin::parse(tin).is_ok(), true);
            assert!(matches!(
                NorwegianTin::parse(tin).unwrap(),
                NorwegianTin::FNumber(_)
            ));
            assert_eq!(
                NorwegianTin::parse(tin).unwrap().get_kind(),
                PersonKind::Normal
            );
        }
    }
    #[test]
    fn test_h_number_fnr() {
        let tins = vec![
            "22517149261",
            "16501622854",
            "10496524328",
            "22492434063",
            "30512441595",
            "26420623894",
            "15446406660",
            "11490936698",
            "31419107320",
            "09486435124",
            "29497811978",
            "24516544984",
            "28449596104",
            "21482101531",
            "09526738303",
            "31473336629",
            "04520606691",
            "07502323096",
            "24484292627",
            "19468111414",
            "07467214181",
            "30418893342",
            "09449633846",
            "17507311458",
            "03477998770",
            "22487823574",
            "02492215786",
            "12512213926",
            "21434290810",
            "03431642904",
            "06490734730",
            "04424032617",
            "16484997113",
            "15424333392",
            "02455927105",
            "19431825034",
            "23453707415",
            "02498505069",
            "14419493563",
            "28519698023",
            "08474914586",
            "29457592246",
            "10467613974",
            "21436941169",
            "09443311979",
            "25431429198",
            "19412412594",
            "01527994698",
            "18483710999",
            "29500441176",
        ];

        for tin in tins {
            assert_eq!(NorwegianTin::parse(tin).is_ok(), true);
            assert!(matches!(
                NorwegianTin::parse(tin).unwrap(),
                NorwegianTin::FNumber(_)
            ));
            assert_eq!(
                NorwegianTin::parse(tin).unwrap().get_kind(),
                PersonKind::HNumber
            );
        }
    }

    #[test]
    fn test_synthetic_dnr() {
        let dnr = vec![
            "70848000149",
            "56865400190",
            "60889201749",
            "70859800961",
            "61915201511",
            "70887100797",
            "47914500210",
            "52909301009",
            "49867500528",
            "55896000267",
            "62847600204",
            "65929301905",
            "45817500521",
            "56878500771",
            "52910875191",
            "44908800521",
            "54815601311",
            "57839200716",
            "44915500312",
            "61887100240",
            "70875300782",
            "47858100948",
            "64857300139",
            "46826902183",
            "45819901183",
            "50828300927",
            "55826700334",
            "50924700936",
            "49878300261",
            "48898400142",
            "46885500463",
            "58925800463",
            "44883300154",
            "41901200279",
            "69915100880",
            "45896500361",
            "59918601168",
            "54928500449",
            "45875301105",
            "64858900172",
            "50866301562",
            "41916500333",
            "57875400381",
            "47869000198",
            "56836700363",
            "63843400118",
            "55918100171",
            "49818302737",
            "50866900512",
            "70924700201",
        ];
        for tin in dnr {
            assert_eq!(NorwegianTin::parse(tin).is_ok(), true);
            assert!(matches!(
                NorwegianTin::parse(tin).unwrap(),
                NorwegianTin::DNumber(_)
            ));
            assert_eq!(
                NorwegianTin::parse(tin).unwrap().get_kind(),
                PersonKind::Synthetic
            );
        }
    }
    #[test]
    fn test_anonymous_fnr() {
        let tins = vec![
            "08639815316",
            "13620315545",
            "26622700351",
            "09624093701",
            "06673803375",
            "17639516431",
            "06721148228",
            "08610428933",
            "01713417137",
            "02611935342",
            "21646200102",
            "05654134997",
            "28616909969",
            "06695929704",
            "03690058307",
            "28711842685",
            "10706095352",
            "19655548648",
            "11671742127",
            "20640081725",
        ];
        for tin in tins {
            assert_eq!(NorwegianTin::parse(tin).is_ok(), true);
            assert!(matches!(
                NorwegianTin::parse(tin).unwrap(),
                NorwegianTin::FNumber(_)
            ));
            assert_eq!(
                NorwegianTin::parse(tin).unwrap().get_kind(),
                PersonKind::Anonymous
            );
        }
    }

    #[test]
    fn test_org_number() {
        let orgs = vec![
            "905661833",
            "085649779",
            "255399985",
            "917766150",
            "406099474",
            "169994803",
            "127412626",
            "661532777",
            "627143508",
            "532464390",
            "625711045",
            "968668056",
            "452238063",
            "882897311",
            "428621840",
            "134511966",
            "565768123",
            "888302964",
            "360559912",
            "635203536",
            "557117245",
            "021244716",
            "331370207",
            "639703991",
            "769676260",
            "084416371",
            "635606576",
            "190537102",
            "945867159",
            "350759131",
            "008303274",
            "515559396",
            "381740196",
            "001686313",
            "977279410",
            "282720493",
            "052965152",
            "310958352",
            "687739515",
            "399784646",
            "389213292",
            "638522837",
            "981000463",
            "973289829",
            "934708431",
            "094979358",
            "003221571",
            "283978958",
            "003437353",
            "123480759",
        ];
        for org in orgs {
            assert!(matches!(
                NorwegianTin::parse(org).unwrap(),
                NorwegianTin::OrgNumber(_)
            ));
        }
    }
    #[test]
    fn test_org_number_invalid() {
        let orgs = vec!["905661834", "085649778", "255399984", "917766151"];
        for org in orgs {
            assert_eq!(
                NorwegianTin::parse(org).unwrap_err(),
                NorwegianTinError::InvalidChecksum
            );
        }
    }

}
