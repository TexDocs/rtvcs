use uuid::Uuid;

#[derive(PartialEq, Clone, Debug)]
pub struct Commit {
    pub id: u32,
    content: CommitContent,
}

impl Commit {
    pub fn new(id: u32, content: CommitContent) -> Commit {
        Commit { id, content }
    }
}

pub fn insert_before(
    remote_commits: Vec<Commit>,
    local_commits: Vec<Commit>,
) -> (Vec<Commit>, Vec<CommitContent>) {
    let document_patch = remote_commits
        .iter()
        .filter_map(|commit| commit.content.shift_forwards_multiple(&local_commits))
        .collect();

    let remote_commits_count = remote_commits.len() as u32;
    let mut previous_local_commits = Vec::new();
    let mut deleted_commits = 0;

    let new_version_vector = local_commits.into_iter().fold(
        Vec::new(),
        |mut previous_shifted_local_commits, commit| {
            let backwards_shifted_commit = commit
                .content
                .shift_backwards_multiple(&previous_local_commits);
            previous_local_commits.push(commit.clone());

            let partial_shifted_commit =
                backwards_shifted_commit.shift_forwards_multiple(&remote_commits);

            if partial_shifted_commit.is_none() {
                deleted_commits += 1;
            }

            if let Some(partial_shifted_commit_content) = partial_shifted_commit {
                let shifted_commit = partial_shifted_commit_content
                    .shift_forwards_multiple(&previous_shifted_local_commits);

                if let Some(shifted_commit_content) = shifted_commit {
                    previous_shifted_local_commits.push(Commit::new(
                        commit.id + remote_commits_count - deleted_commits,
                        shifted_commit_content,
                    ));
                }
            }

            previous_shifted_local_commits
        },
    );

    (new_version_vector, document_patch)
}

#[derive(PartialEq, Clone, Debug)]
pub enum CommitContent {
    InsertTextCommit(InsertTextCommit),
    DeleteTextCommit(DeleteTextCommit),
    AddFileCommit(AddFileCommit),
    DeleteFileCommit(DeleteFileCommit),
}

impl CommitContent {
    pub fn shift_forwards_multiple(&self, other_commits: &Vec<Commit>) -> Option<CommitContent> {
        let mut current_commit = self.clone();
        for other_commit in other_commits.iter() {
            match current_commit.shift_forwards(&other_commit.content) {
                Some(shifted_commit) => {
                    current_commit = shifted_commit;
                }
                _ => {
                    return None;
                }
            };
        }
        Some(current_commit.clone())
    }

    pub fn shift_forwards(&self, other_commit: &CommitContent) -> Option<CommitContent> {
        match (self, other_commit) {
            (
                &CommitContent::InsertTextCommit(ref commit),
                &CommitContent::InsertTextCommit(ref other_commit),
            ) => {
                if commit.file != other_commit.file {
                    Some(CommitContent::InsertTextCommit(commit.clone()))
                } else if commit == other_commit {
                    None
                } else if commit.location < other_commit.location {
                    Some(CommitContent::InsertTextCommit(commit.clone()))
                } else {
                    Some(CommitContent::InsertTextCommit(
                        commit.clone().shifted_by(other_commit.text.len() as i64),
                    ))
                }
            }
            (
                &CommitContent::InsertTextCommit(ref commit),
                &CommitContent::DeleteTextCommit(ref other_commit),
            ) => {
                if commit.file != other_commit.file {
                    Some(CommitContent::InsertTextCommit(commit.clone()))
                } else if commit.location < other_commit.location {
                    Some(CommitContent::InsertTextCommit(commit.clone()))
                } else if other_commit.max_location() <= commit.location {
                    Some(CommitContent::InsertTextCommit(
                        commit.shifted_by(-other_commit.length),
                    ))
                } else {
                    Some(CommitContent::InsertTextCommit(
                        commit.clone().with_location(other_commit.location),
                    ))
                }
            }
            (&CommitContent::InsertTextCommit(ref commit), &CommitContent::AddFileCommit(_)) => {
                Some(CommitContent::InsertTextCommit(commit.clone()))
            }
            (
                &CommitContent::InsertTextCommit(ref commit),
                &CommitContent::DeleteFileCommit(ref other_commit),
            ) => {
                if commit.file == other_commit.file {
                    None
                } else {
                    Some(CommitContent::InsertTextCommit(commit.clone()))
                }
            }
            (
                &CommitContent::DeleteTextCommit(ref commit),
                &CommitContent::InsertTextCommit(ref other_commit),
            ) => {
                if commit.file != other_commit.file {
                    Some(CommitContent::DeleteTextCommit(commit.clone()))
                } else if commit.location >= other_commit.location {
                    Some(CommitContent::DeleteTextCommit(
                        commit.clone().shifted_by(other_commit.text.len() as i64),
                    ))
                } else if commit.max_location() <= other_commit.location {
                    Some(CommitContent::DeleteTextCommit(commit.clone()))
                } else {
                    // TODO Evalute other variants
                    None
                }
            }
            (
                &CommitContent::DeleteTextCommit(ref commit),
                &CommitContent::DeleteTextCommit(ref other_commit),
            ) => {
                if commit.file != other_commit.file {
                    Some(CommitContent::DeleteTextCommit(commit.clone()))
                } else if commit == other_commit {
                    None
                } else if commit.max_location() <= other_commit.location {
                    Some(CommitContent::DeleteTextCommit(commit.clone()))
                } else if other_commit.max_location() <= commit.location {
                    Some(CommitContent::DeleteTextCommit(
                        commit.clone().shifted_by(-other_commit.length),
                    ))
                } else if commit.location < other_commit.location {
                    // TODO Evalute other variants
                    None
                } else {
                    // TODO Evalute other variants
                    None
                }
            }
            (&CommitContent::DeleteTextCommit(ref commit), &CommitContent::AddFileCommit(_)) => {
                Some(CommitContent::DeleteTextCommit(commit.clone()))
            }
            (
                &CommitContent::DeleteTextCommit(ref commit),
                &CommitContent::DeleteFileCommit(ref other_commit),
            ) => {
                if commit.file == other_commit.file {
                    None
                } else {
                    Some(CommitContent::DeleteTextCommit(commit.clone()))
                }
            }
            (&CommitContent::AddFileCommit(ref commit), &CommitContent::InsertTextCommit(_)) => {
                Some(CommitContent::AddFileCommit(commit.clone()))
            }
            (&CommitContent::AddFileCommit(ref commit), &CommitContent::DeleteTextCommit(_)) => {
                Some(CommitContent::AddFileCommit(commit.clone()))
            }
            (
                &CommitContent::AddFileCommit(ref commit),
                &CommitContent::AddFileCommit(ref other_commit),
            ) => {
                if commit == other_commit {
                    None
                } else {
                    Some(CommitContent::AddFileCommit(commit.clone()))
                }
            }
            (&CommitContent::AddFileCommit(ref commit), &CommitContent::DeleteFileCommit(_)) => {
                Some(CommitContent::AddFileCommit(commit.clone()))
            }
            (&CommitContent::DeleteFileCommit(ref commit), &CommitContent::InsertTextCommit(_)) => {
                Some(CommitContent::DeleteFileCommit(commit.clone()))
            }
            (&CommitContent::DeleteFileCommit(ref commit), &CommitContent::DeleteTextCommit(_)) => {
                Some(CommitContent::DeleteFileCommit(commit.clone()))
            }
            (&CommitContent::DeleteFileCommit(ref commit), &CommitContent::AddFileCommit(_)) => {
                Some(CommitContent::DeleteFileCommit(commit.clone()))
            }
            (
                &CommitContent::DeleteFileCommit(ref commit),
                &CommitContent::DeleteFileCommit(ref other_commit),
            ) => {
                if commit == other_commit {
                    None
                } else {
                    Some(CommitContent::DeleteFileCommit(commit.clone()))
                }
            }
        }
    }

    pub fn shift_backwards_multiple(&self, other_commits: &Vec<Commit>) -> CommitContent {
        let mut current_commit = self.clone();
        for other_commit in other_commits.iter().rev() {
            current_commit = current_commit.shift_backwards(&other_commit.content);
        }
        current_commit.clone()
    }

    pub fn shift_backwards(&self, other_commit: &CommitContent) -> CommitContent {
        match (self, other_commit) {
            (
                &CommitContent::InsertTextCommit(ref commit),
                &CommitContent::InsertTextCommit(ref other_commit),
            ) => {
                if commit.file != other_commit.file || commit == other_commit
                    || commit.location < other_commit.location
                {
                    CommitContent::InsertTextCommit(commit.clone())
                } else {
                    CommitContent::InsertTextCommit(
                        commit.clone().shifted_by(-(other_commit.text.len() as i64)),
                    )
                }
            }
            (
                &CommitContent::InsertTextCommit(ref commit),
                &CommitContent::DeleteTextCommit(ref other_commit),
            ) => {
                if commit.file != other_commit.file || commit.location < other_commit.location {
                    CommitContent::InsertTextCommit(commit.clone())
                } else if other_commit.max_location() <= commit.location - other_commit.length {
                    CommitContent::InsertTextCommit(commit.clone().shifted_by(other_commit.length))
                } else {
                    CommitContent::InsertTextCommit(
                        commit.clone().with_location(other_commit.location),
                    )
                }
            }
            (&CommitContent::InsertTextCommit(ref commit), &CommitContent::AddFileCommit(_)) => {
                CommitContent::InsertTextCommit(commit.clone())
            }
            (&CommitContent::InsertTextCommit(ref commit), &CommitContent::DeleteFileCommit(_)) => {
                CommitContent::InsertTextCommit(commit.clone())
            }
            (
                &CommitContent::DeleteTextCommit(ref commit),
                &CommitContent::InsertTextCommit(ref other_commit),
            ) => {
                if commit.location - (other_commit.text.len() as i64) >= other_commit.location {
                    CommitContent::DeleteTextCommit(
                        commit.clone().shifted_by(-(other_commit.text.len() as i64)),
                    )
                } else {
                    CommitContent::DeleteTextCommit(commit.clone())
                }
            }
            (
                &CommitContent::DeleteTextCommit(ref commit),
                &CommitContent::DeleteTextCommit(ref other_commit),
            ) => {
                if other_commit.max_location() <= commit.location - other_commit.length {
                    CommitContent::DeleteTextCommit(commit.clone().shifted_by(other_commit.length))
                } else {
                    CommitContent::DeleteTextCommit(commit.clone())
                }
            }
            (&CommitContent::DeleteTextCommit(ref commit), &CommitContent::AddFileCommit(_)) => {
                CommitContent::DeleteTextCommit(commit.clone())
            }
            (&CommitContent::DeleteTextCommit(ref commit), &CommitContent::DeleteFileCommit(_)) => {
                CommitContent::DeleteTextCommit(commit.clone())
            }
            (&CommitContent::AddFileCommit(ref commit), _) => {
                CommitContent::AddFileCommit(commit.clone())
            }
            (&CommitContent::DeleteFileCommit(ref commit), _) => {
                CommitContent::DeleteFileCommit(commit.clone())
            }
        }
    }
}

#[derive(PartialEq, Clone, Debug)]
pub struct InsertTextCommit {
    pub location: i64,
    pub text: String,
    pub file: Uuid,
}

impl InsertTextCommit {
    pub fn max_location(&self) -> i64 {
        self.location + self.text.len() as i64
    }

    fn with_location(self, location: i64) -> InsertTextCommit {
        InsertTextCommit {
            location: location,
            text: self.text,
            file: self.file,
        }
    }

    fn shifted_by(&self, shift: i64) -> InsertTextCommit {
        self.clone().with_location(self.location + shift)
    }
}

#[derive(PartialEq, Clone, Debug)]
pub struct DeleteTextCommit {
    pub location: i64,
    pub length: i64,
    pub file: Uuid,
}

impl DeleteTextCommit {
    pub fn max_location(&self) -> i64 {
        self.location + self.length
    }

    fn with_location(&self, location: i64) -> DeleteTextCommit {
        DeleteTextCommit {
            location: location,
            length: self.length,
            file: self.file,
        }
    }

    fn shifted_by(&self, shift: i64) -> DeleteTextCommit {
        self.with_location(self.location + shift)
    }
}

#[derive(PartialEq, Clone, Debug)]
pub struct AddFileCommit {
    pub name: String,
    pub content: Option<Vec<u8>>,
    pub file: Uuid,
}

#[derive(PartialEq, Clone, Debug)]
pub struct DeleteFileCommit {
    pub name: String,
    pub file: Uuid,
}

#[test]
fn shift_commit() {
    let local_commits = vec![
        Commit::new(
            0,
            CommitContent::InsertTextCommit(InsertTextCommit {
                location: 0,
                text: String::from("Hello World"),
                file: Uuid::nil(),
            }),
        ),
        Commit::new(
            1,
            CommitContent::InsertTextCommit(InsertTextCommit {
                location: 11,
                text: String::from(" This is a text."),
                file: Uuid::nil(),
            }),
        ),
    ];

    let remote_commits = vec![
        Commit::new(
            0,
            CommitContent::InsertTextCommit(InsertTextCommit {
                location: 5,
                text: String::from("Text at 5"),
                file: Uuid::nil(),
            }),
        ),
        Commit::new(
            1,
            CommitContent::InsertTextCommit(InsertTextCommit {
                location: 9,
                text: String::from(" Test with 9."),
                file: Uuid::nil(),
            }),
        ),
    ];

    let shifted_commits = insert_before(remote_commits.clone(), local_commits);

    for commit in shifted_commits.1 {
        println!("{:?}", commit);
    }

    println!("----------------");
    for commit in remote_commits {
        println!("{:?}", commit);
    }
    for commit in shifted_commits.0 {
        println!("{:?}", commit);
    }

    assert_eq!(2 + 2, 4);
}
