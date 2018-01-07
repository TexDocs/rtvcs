use uuid::Uuid;

pub enum Commit {
    InsertTextCommit(InsertTextCommit),
    DeleteTextCommit(DeleteTextCommit),
    AddFileCommit(AddFileCommit),
    DeleteFileCommit(DeleteFileCommit),
}

impl Commit {
    pub fn shift_along_commits(self, other_commits: Vec<Commit>) -> Option<Commit> {
        let mut current_commit = self;
        for other_commit in other_commits {
            match current_commit.shift_along_commit(other_commit) {
                Some(shifted_commit) => {
                    current_commit = shifted_commit;
                }
                _ => {
                    return None;
                }
            };
        }
        Some(current_commit)
    }

    pub fn shift_along_commit(self, other_commit: Commit) -> Option<Commit> {
        match (self, other_commit) {
            (Commit::InsertTextCommit(commit), Commit::InsertTextCommit(other_commit)) => {
                if commit == other_commit {
                    None
                } else if commit.location <= other_commit.location {
                    Some(Commit::InsertTextCommit(commit))
                } else {
                    Some(Commit::InsertTextCommit(
                        commit.shifted_by(other_commit.text.len() as i64),
                    ))
                }
            }
            (Commit::InsertTextCommit(commit), Commit::DeleteTextCommit(other_commit)) => {
                if commit.location <= other_commit.location {
                    Some(Commit::InsertTextCommit(commit))
                } else if other_commit.max_location() <= commit.location {
                    Some(Commit::InsertTextCommit(
                        commit.shifted_by(-other_commit.length),
                    ))
                } else {
                    Some(Commit::InsertTextCommit(
                        commit.with_location(other_commit.location),
                    ))
                }
            }
            (Commit::InsertTextCommit(commit), Commit::AddFileCommit(_)) => {
                Some(Commit::InsertTextCommit(commit))
            }
            (Commit::InsertTextCommit(commit), Commit::DeleteFileCommit(other_commit)) => {
                if commit.file == other_commit.file {
                    None
                } else {
                    Some(Commit::InsertTextCommit(commit))
                }
            }
            (Commit::DeleteTextCommit(commit), Commit::InsertTextCommit(other_commit)) => {
                if commit.location >= other_commit.location {
                    Some(Commit::DeleteTextCommit(
                        commit.shifted_by(other_commit.text.len() as i64),
                    ))
                } else if commit.max_location() <= other_commit.location {
                    Some(Commit::DeleteTextCommit(commit))
                } else {
                    Some(Commit::DeleteTextCommit(
                        commit.with_lenght(other_commit.location - commit.location),
                    ))
                }
            }
            (Commit::DeleteTextCommit(commit), Commit::DeleteTextCommit(other_commit)) => {
                if commit == other_commit {
                    None
                } else if commit.max_location() <= other_commit.location {
                    Some(Commit::DeleteTextCommit(commit))
                } else if other_commit.max_location() <= commit.location {
                    Some(Commit::DeleteTextCommit(
                        commit.shifted_by(-other_commit.length),
                    ))
                } else if commit.location <= other_commit.location {
                    Some(Commit::DeleteTextCommit(
                        commit.with_lenght(other_commit.location - commit.location),
                    ))
                } else {
                    let overlapping_length = other_commit.max_location() - commit.location;
                    Some(Commit::DeleteTextCommit(
                        commit
                            .with_location(other_commit.max_location())
                            .with_lenght(commit.length - overlapping_length),
                    ))
                }
            }
            (Commit::DeleteTextCommit(commit), Commit::AddFileCommit(_)) => {
                Some(Commit::DeleteTextCommit(commit))
            }
            (Commit::DeleteTextCommit(commit), Commit::DeleteFileCommit(other_commit)) => {
                if commit.file == other_commit.file {
                    None
                } else {
                    Some(Commit::DeleteTextCommit(commit))
                }
            }
            (Commit::AddFileCommit(commit), Commit::InsertTextCommit(_)) => {
                Some(Commit::AddFileCommit(commit))
            }
            (Commit::AddFileCommit(commit), Commit::DeleteTextCommit(_)) => {
                Some(Commit::AddFileCommit(commit))
            }
            (Commit::AddFileCommit(commit), Commit::AddFileCommit(other_commit)) => {
                if commit == other_commit {
                    None
                } else {
                    Some(Commit::AddFileCommit(commit))
                }
            }
            (Commit::AddFileCommit(commit), Commit::DeleteFileCommit(_)) => {
                Some(Commit::AddFileCommit(commit))
            }
            (Commit::DeleteFileCommit(commit), Commit::InsertTextCommit(_)) => {
                Some(Commit::DeleteFileCommit(commit))
            }
            (Commit::DeleteFileCommit(commit), Commit::DeleteTextCommit(_)) => {
                Some(Commit::DeleteFileCommit(commit))
            }
            (Commit::DeleteFileCommit(commit), Commit::AddFileCommit(_)) => {
                Some(Commit::DeleteFileCommit(commit))
            }
            (Commit::DeleteFileCommit(commit), Commit::DeleteFileCommit(other_commit)) => {
                if commit == other_commit {
                    None
                } else {
                    Some(Commit::DeleteFileCommit(commit))
                }
            }
        }
    }
}

#[derive(PartialEq, Clone)]
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

#[derive(PartialEq)]
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

    fn with_lenght(&self, length: i64) -> DeleteTextCommit {
        DeleteTextCommit {
            location: self.location,
            length: length,
            file: self.file,
        }
    }
}

#[derive(PartialEq)]
pub struct AddFileCommit {
    pub name: String,
    pub content: Option<Vec<u8>>,
    pub file: Uuid,
}

#[derive(PartialEq)]
pub struct DeleteFileCommit {
    pub name: String,
    pub file: Uuid,
}
