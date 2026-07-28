use uuid::Uuid;

/// The longest a stored filename may be. Keys have a limit of their own in S3, and a 900
/// character filename is nobody's real filename.
const MAX_FILENAME: usize = 100;

/// Builds the key a user's file is stored under.
///
/// Every key sits under a prefix belonging to its owner, which is what makes ownership a
/// property of the key itself rather than something to look up. A random component means two
/// uploads of `photo.jpg` do not overwrite one another.
pub fn for_user(owner: Uuid, filename: &str) -> String {
    format!("uploads/{owner}/{}-{}", Uuid::new_v4(), sanitize(filename))
}

/// Whether a key belongs to a user.
///
/// Checked against the whole prefix including the trailing slash, so one user id cannot be a
/// prefix of another's, and keys that try to climb out with `..` are refused outright.
pub fn belongs_to(key: &str, owner: Uuid) -> bool {
    !key.contains("..") && key.starts_with(&format!("uploads/{owner}/"))
}

/// Reduces a filename to something safe to put in a key.
///
/// A key is not a path, but it is routinely turned back into one by whatever downloads it, so
/// slashes have to go. Anything unexpected becomes a dash rather than being dropped, so two
/// different names cannot quietly collapse into one.
///
/// Runs of dots collapse to a single dot, and leading dots go entirely. Both matter more than
/// they look: a name of `../../etc/passwd` would otherwise keep its `..`, and a key containing
/// `..` is refused by `belongs_to`, so the upload would succeed and then be unreachable.
fn sanitize(filename: &str) -> String {
    let mut cleaned = String::with_capacity(filename.len().min(MAX_FILENAME));

    for character in filename.chars() {
        let character = match character {
            'a'..='z' | 'A'..='Z' | '0'..='9' | '.' | '-' | '_' => character,
            _ => '-',
        };

        let after_a_dot = cleaned.ends_with('.');

        if character == '.' && (after_a_dot || cleaned.is_empty()) {
            continue;
        }

        cleaned.push(character);

        if cleaned.chars().count() == MAX_FILENAME {
            break;
        }
    }

    // A name that was nothing but punctuation, or nothing at all, still needs to be something.
    if cleaned.is_empty() {
        "file".to_owned()
    } else {
        cleaned
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_key_lives_under_its_owner() {
        let owner = Uuid::new_v4();
        let key = for_user(owner, "photo.jpg");

        assert!(key.starts_with(&format!("uploads/{owner}/")));
        assert!(key.ends_with("-photo.jpg"));
    }

    #[test]
    fn two_uploads_of_one_name_do_not_collide() {
        let owner = Uuid::new_v4();

        assert_ne!(for_user(owner, "photo.jpg"), for_user(owner, "photo.jpg"));
    }

    #[test]
    fn traversal_does_not_survive_a_filename() {
        let owner = Uuid::new_v4();
        let key = for_user(owner, "../../etc/passwd");

        assert!(!key.contains(".."));
        assert!(key.starts_with(&format!("uploads/{owner}/")));
        assert_eq!(key.matches('/').count(), 2);

        // And the key it produced has to be one its owner can still use afterwards.
        assert!(belongs_to(&key, owner));
    }

    /// Dots in the middle of a real name are ordinary, and only runs of them are suspicious.
    #[test]
    fn an_ordinary_name_keeps_its_shape() {
        let key = for_user(Uuid::new_v4(), "report.final.v2.pdf");

        assert!(key.ends_with("-report.final.v2.pdf"), "{key}");
    }

    #[test]
    fn a_name_that_is_only_punctuation_still_gets_one() {
        let owner = Uuid::new_v4();

        assert!(for_user(owner, "...").ends_with("-file"));
        assert!(for_user(owner, "").ends_with("-file"));
    }

    #[test]
    fn a_long_name_is_cut_short() {
        let owner = Uuid::new_v4();
        let key = for_user(owner, &"a".repeat(500));

        assert!(key.len() < 200, "{key}");
    }

    #[test]
    fn an_owner_can_only_claim_their_own_keys() {
        let ada = Uuid::new_v4();
        let grace = Uuid::new_v4();
        let key = for_user(ada, "photo.jpg");

        assert!(belongs_to(&key, ada));
        assert!(!belongs_to(&key, grace));
    }

    /// A key must not be claimable by pointing at somebody else's prefix from inside your own.
    #[test]
    fn climbing_out_of_a_prefix_is_refused() {
        let ada = Uuid::new_v4();
        let key = format!("uploads/{ada}/../{}/photo.jpg", Uuid::new_v4());

        assert!(!belongs_to(&key, ada));
    }

    #[test]
    fn another_prefix_entirely_is_refused() {
        let ada = Uuid::new_v4();

        assert!(!belongs_to("secrets/database-dump.sql", ada));
        assert!(!belongs_to(&format!("uploads/{ada}"), ada));
    }
}
