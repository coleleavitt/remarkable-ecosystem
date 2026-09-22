#![allow(dead_code)]
//! Pattern scanning for binary analysis

use crate::binary::XochitlBinary;

/// A pattern to scan for
#[derive(Debug, Clone)]
pub struct ScanTarget {
    pub name: &'static str,
    pub description: &'static str,
    pub pattern: &'static [u8],
    pub mask: Option<&'static [u8]>,
}

/// Built-in scan targets for xochitl analysis
pub static SCAN_TARGETS: &[ScanTarget] = &[
    ScanTarget {
        name: "InsecureSettings",
        description: "Hidden developer settings module",
        pattern: b"insecureSettings\x00",
        mask: None,
    },
    ScanTarget {
        name: "FeatureToggles",
        description: "Feature toggle system",
        pattern: b"FeatureToggles\x00",
        mask: None,
    },
    ScanTarget {
        name: "TelemetryV2",
        description: "Telemetry collection module",
        pattern: b"telemetryv2\x00",
        mask: None,
    },
    ScanTarget {
        name: "Analytics",
        description: "Analytics session module",
        pattern: b"analytics-session\x00",
        mask: None,
    },
    ScanTarget {
        name: "CloudSync",
        description: "Cloud sync endpoint",
        pattern: b"my.remarkable.com\x00",
        mask: None,
    },
    ScanTarget {
        name: "TectonicAPI",
        description: "Tectonic sync API",
        pattern: b"tectonic.remarkable.com\x00",
        mask: None,
    },
    ScanTarget {
        name: "Memfault",
        description: "Memfault crash reporting",
        pattern: b"memfault\x00",
        mask: None,
    },
    ScanTarget {
        name: "BootSplash",
        description: "Boot splash module",
        pattern: b"bootsplash\x00",
        mask: None,
    },
    ScanTarget {
        name: "BetaProgram",
        description: "Beta program enrollment",
        pattern: b"BetaProgram\x00",
        mask: None,
    },
    ScanTarget {
        name: "DeveloperPassword",
        description: "Developer mode password",
        pattern: b"developerPassword\x00",
        mask: None,
    },
    ScanTarget {
        name: "DebugGrid",
        description: "Debug grid overlay",
        pattern: b"debugGrid\x00",
        mask: None,
    },
    ScanTarget {
        name: "Experimental",
        description: "Experimental features",
        pattern: b"Experimental\x00",
        mask: None,
    },
    ScanTarget {
        name: "RetailDemo",
        description: "Retail demo mode",
        pattern: b"retailDemoIndicator\x00",
        mask: None,
    },
    ScanTarget {
        name: "ScreenShare",
        description: "Screen share feature",
        pattern: b"screenshare\x00",
        mask: None,
    },
    ScanTarget {
        name: "WebRTC",
        description: "WebRTC signaling",
        pattern: b"webrtc\x00",
        mask: None,
    },
    ScanTarget {
        name: "MQTT",
        description: "MQTT messaging",
        pattern: b"mqtt\x00",
        mask: None,
    },
    ScanTarget {
        name: "ThirdPartySync",
        description: "Third-party sync access check",
        pattern: b"hasSyncThirdPartyAccess\x00",
        mask: None,
    },
    ScanTarget {
        name: "OmahaUpdate",
        description: "OTA update system",
        pattern: b"omahaupdate\x00",
        mask: None,
    },
];

/// Pattern scanner for xochitl binaries
pub struct PatternScanner<'a> {
    data: &'a [u8],
}

impl<'a> PatternScanner<'a> {
    /// Create a new scanner for a binary
    pub fn new(binary: &'a XochitlBinary) -> Self {
        Self { data: binary.data() }
    }

    /// Find a scan target pattern
    pub fn find_pattern(&self, target: &ScanTarget) -> Option<usize> {
        if let Some(mask) = target.mask {
            self.find_masked(target.pattern, mask)
        } else {
            self.find_raw(target.pattern)
        }
    }

    /// Find raw bytes without mask
    pub fn find_raw(&self, pattern: &[u8]) -> Option<usize> {
        self.data.windows(pattern.len()).position(|w| w == pattern)
    }

    /// Find all occurrences of a pattern
    pub fn find_all(&self, pattern: &[u8]) -> Vec<usize> {
        let mut results = Vec::new();
        let mut pos = 0;

        while pos < self.data.len().saturating_sub(pattern.len()) {
            if &self.data[pos..pos + pattern.len()] == pattern {
                results.push(pos);
                pos += 1; // Allow overlapping matches
            } else {
                pos += 1;
            }
        }

        results
    }

    /// Find bytes with a mask (0xFF = must match, 0x00 = wildcard)
    pub fn find_masked(&self, pattern: &[u8], mask: &[u8]) -> Option<usize> {
        if pattern.len() != mask.len() {
            return None;
        }

        'outer: for (i, window) in self.data.windows(pattern.len()).enumerate() {
            for (j, (&p, &m)) in pattern.iter().zip(mask.iter()).enumerate() {
                if m == 0xFF && window[j] != p {
                    continue 'outer;
                }
            }
            return Some(i);
        }

        None
    }

    /// Find a regex pattern in strings
    pub fn find_regex(&self, pattern: &str) -> Vec<(usize, String)> {
        let re = match regex::bytes::Regex::new(pattern) {
            Ok(r) => r,
            Err(_) => return Vec::new(),
        };

        re.find_iter(self.data)
            .map(|m| (m.start(), String::from_utf8_lossy(m.as_bytes()).to_string()))
            .collect()
    }

    /// Extract null-terminated string at offset
    pub fn string_at(&self, offset: usize) -> Option<String> {
        if offset >= self.data.len() {
            return None;
        }

        let end = self.data[offset..]
            .iter()
            .position(|&b| b == 0)
            .unwrap_or(256.min(self.data.len() - offset));

        String::from_utf8(self.data[offset..offset + end].to_vec()).ok()
    }

    /// Scan for ARM32 function prologues
    pub fn find_function_prologues(&self) -> Vec<usize> {
        let mut results = Vec::new();

        // ARM32 Thumb-2 push {r4-r7, lr} = 0xB5F0
        // ARM32 Thumb-2 push {r3-r7, lr} = 0xB5F8
        let patterns = [
            &[0xF0, 0xB5][..], // push {r4-r7, lr}
            &[0xF8, 0xB5][..], // push {r3-r7, lr}
            &[0x70, 0xB5][..], // push {r4-r6, lr}
            &[0x80, 0xB5][..], // push {r7, lr}
            &[0x00, 0xB5][..], // push {lr}
        ];

        for pattern in patterns {
            for (i, window) in self.data.windows(2).enumerate() {
                if window == pattern {
                    results.push(i);
                }
            }
        }

        results.sort();
        results.dedup();
        results
    }

    /// Find cross-references to an address (simple heuristic)
    pub fn find_xrefs(&self, target: u32) -> Vec<usize> {
        let target_bytes = target.to_le_bytes();
        self.find_all(&target_bytes)
    }
}

/// ARM instruction decoder (simplified)
pub mod arm {
    /// Decode a Thumb-2 instruction
    pub fn decode_thumb2(data: &[u8], offset: usize) -> Option<String> {
        if offset + 2 > data.len() {
            return None;
        }

        let hw1 = u16::from_le_bytes([data[offset], data[offset + 1]]);

        // Check if this is a 32-bit instruction
        let is_32bit = (hw1 >> 11) >= 0x1D;

        if is_32bit {
            if offset + 4 > data.len() {
                return None;
            }
            let hw2 = u16::from_le_bytes([data[offset + 2], data[offset + 3]]);
            decode_thumb2_32(hw1, hw2)
        } else {
            decode_thumb2_16(hw1)
        }
    }

    fn decode_thumb2_16(hw: u16) -> Option<String> {
        match hw >> 10 {
            0b010000 => {
                // Data-processing
                let op = (hw >> 6) & 0xF;
                let rm = (hw >> 3) & 0x7;
                let rd = hw & 0x7;
                let op_name = match op {
                    0 => "AND",
                    1 => "EOR",
                    2 => "LSL",
                    3 => "LSR",
                    4 => "ASR",
                    5 => "ADC",
                    6 => "SBC",
                    7 => "ROR",
                    8 => "TST",
                    9 => "RSB",
                    10 => "CMP",
                    11 => "CMN",
                    12 => "ORR",
                    13 => "MUL",
                    14 => "BIC",
                    15 => "MVN",
                    _ => return None,
                };
                Some(format!("{} R{}, R{}", op_name, rd, rm))
            }
            0b101101 if (hw >> 9) & 1 == 1 => {
                // PUSH
                let regs = hw & 0xFF;
                let lr = (hw >> 8) & 1 == 1;
                let mut reg_list = String::new();
                for i in 0..8 {
                    if (regs >> i) & 1 == 1 {
                        if !reg_list.is_empty() {
                            reg_list.push_str(", ");
                        }
                        reg_list.push_str(&format!("R{}", i));
                    }
                }
                if lr {
                    if !reg_list.is_empty() {
                        reg_list.push_str(", ");
                    }
                    reg_list.push_str("LR");
                }
                Some(format!("PUSH {{{}}}", reg_list))
            }
            0b10111 if (hw & 0x0F00) == 0x0F00 => {
                // NOP
                Some("NOP".to_string())
            }
            _ => {
                // Unknown or complex instruction
                Some(format!("DW 0x{:04X}", hw))
            }
        }
    }

    fn decode_thumb2_32(hw1: u16, hw2: u16) -> Option<String> {
        let op1 = (hw1 >> 11) & 0x3;
        let op2 = (hw1 >> 4) & 0x7F;

        match (op1, op2 >> 5) {
            (1, 0b01) => {
                // Branch with link
                let s = (hw1 >> 10) & 1;
                let imm10 = hw1 & 0x3FF;
                let j1 = (hw2 >> 13) & 1;
                let j2 = (hw2 >> 11) & 1;
                let imm11 = hw2 & 0x7FF;

                let i1 = !((j1 ^ s) & 1);
                let i2 = !((j2 ^ s) & 1);

                let offset = ((s as i32) << 24)
                    | ((i1 as i32) << 23)
                    | ((i2 as i32) << 22)
                    | ((imm10 as i32) << 12)
                    | ((imm11 as i32) << 1);

                let offset = if s == 1 {
                    offset | 0xFE000000_u32 as i32
                } else {
                    offset
                };

                Some(format!("BL #0x{:X}", offset))
            }
            _ => Some(format!("DW 0x{:04X}{:04X}", hw1, hw2)),
        }
    }
}

#[cfg(test)]
mod tests {
    

    #[test]
    fn test_find_raw() {
        let data = b"hello world hello";
        struct MockBinary(Vec<u8>);
        impl MockBinary {
            fn data(&self) -> &[u8] {
                &self.0
            }
        }

        // Direct test without XochitlBinary
        assert!(data.windows(5).position(|w| w == b"hello").is_some());
        assert!(data.windows(5).position(|w| w == b"xxxxx").is_none());
    }
}
