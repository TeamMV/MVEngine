use crate::graphics::comp::parse::lexer::{MRFLexer, MRFToken};
use crate::graphics::comp::parse::rig::{ParsedPart, Parsed, ParsedRig, ParsedBone, BoneStart, ParsedJoint};

pub struct MRFParser;

impl MRFParser {
    pub fn parse(s: &str) -> Result<ParsedRig, String> {
        let mut parts = Parsed::new();
        let mut bones = Parsed::new();
        let mut joints = Parsed::new();
        
        let mut lexer = MRFLexer::new(s);
        let mut next = lexer.next();
        while let Some(n) = next {
            match n {
                MRFToken::EOF => {
                    break; //very dirty fix but noone cares and it works
                }
                MRFToken::Parts => {
                    parts = Self::parse_parts(&mut lexer)?;
                }
                MRFToken::Bones => {
                    bones = Self::parse_bones(&mut lexer)?;
                }
                MRFToken::Joints => {
                    joints = Self::parse_joints(&mut lexer, &bones)?;
                }
                MRFToken::Attach => {
                    Self::parse_attachments(&mut lexer, &bones, &mut parts)?;
                }
                MRFToken::Error(err) => return Err(err),
                _ => return Err(format!("Unexpected Token: {n:?}"))
            };
            
            next = lexer.next();
        }
        
        Ok(ParsedRig {
            parts,
            bones,
            joints,
        })
    }
    
    fn parse_parts(lexer: &mut MRFLexer) -> Result<Parsed<ParsedPart>, String> {
        let mut parts = Parsed::new();
        
        let mut next = lexer.next_some()?;
        while let MRFToken::Ident(_) = &next {
            lexer.putback(next);
            let part = Self::parse_part(lexer)?;
            parts.map.insert(part.name.clone(), part);
            next = lexer.next_some()?;
        }
        lexer.putback(next);
        
        Ok(parts)
    }
    
    fn parse_part(lexer: &mut MRFLexer) -> Result<ParsedPart, String> {
        let name = lexer.next_ident()?;
        lexer.next_token(MRFToken::Colon)?;
        let size = lexer.next_vec2()?;
        lexer.next_token(MRFToken::Anchor)?;
        let anchor = lexer.next_vec2()?;
        Ok(ParsedPart {
            name,
            size,
            anchor,
            bone: None,
        })
    }
    
    fn parse_bones(lexer: &mut MRFLexer) -> Result<Parsed<ParsedBone>, String> {
        let mut bones = Parsed::new();

        let mut next = lexer.next_some()?;
        while let MRFToken::Ident(_) = &next {
            lexer.putback(next);
            let bone = Self::parse_bone(lexer)?;
            bones.map.insert(bone.name.clone(), bone);
            next = lexer.next_some()?;
        }
        lexer.putback(next);

        Ok(bones)
    }
    
    fn parse_bone(lexer: &mut MRFLexer) -> Result<ParsedBone, String> {
        let name = lexer.next_ident()?;
        lexer.next_token(MRFToken::Colon)?;
        let start = lexer.next_some()?;
        let start = match start {
            MRFToken::Ident(other) => BoneStart::Other(other),
            MRFToken::Vec2(point) => BoneStart::Point(point),
            _ => return Err(format!("Unexpected Token: {start:?}"))
        };
        lexer.next_token(MRFToken::Selector)?;
        let end = lexer.next_vec2()?;
        Ok(ParsedBone {
            name,
            start,
            end,
        })
    }

    fn parse_joints(lexer: &mut MRFLexer, bones: &Parsed<ParsedBone>) -> Result<Parsed<ParsedJoint>, String> {
        let mut joints = Parsed::new();

        let mut next = lexer.next_some()?;
        while let MRFToken::Ident(_) = &next {
            lexer.putback(next);
            let joint = Self::parse_joint(lexer, bones)?;
            joints.map.insert(joint.name.clone(), joint);
            next = lexer.next_some()?;
        }
        lexer.putback(next);

        Ok(joints)
    }

    fn parse_joint(lexer: &mut MRFLexer, bones: &Parsed<ParsedBone>) -> Result<ParsedJoint, String> {
        let name = lexer.next_ident()?;
        lexer.next_token(MRFToken::Colon)?;
        let bone1 = lexer.next_ident()?;
        lexer.next_token(MRFToken::Selector)?;
        let bone2 = lexer.next_ident()?;
        
        bones.verify(&bone1)?;
        bones.verify(&bone2)?;
        
        Ok(ParsedJoint {
            name,
            bone1,
            bone2,
        })
    }
    
    fn parse_attachments(lexer: &mut MRFLexer, bones: &Parsed<ParsedBone>, parts: &mut Parsed<ParsedPart>) -> Result<(), String> {
        let mut next = lexer.next_some()?;
        while let MRFToken::Ident(_) = &next {
            lexer.putback(next);
            Self::parse_attachment(lexer, bones, parts)?;
            next = lexer.next_some()?;
        }
        lexer.putback(next);
        Ok(())
    }

    fn parse_attachment(lexer: &mut MRFLexer, bones: &Parsed<ParsedBone>, parts: &mut Parsed<ParsedPart>) -> Result<(), String> {
        let part = lexer.next_ident()?;
        lexer.next_token(MRFToken::Selector)?;
        let bone = lexer.next_ident()?;
        
        bones.verify(&bone)?;
        let part = parts.find_mut(&part)?;
        part.bone = Some(bone);
        Ok(())
    }
}