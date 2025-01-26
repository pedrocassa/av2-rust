#![cfg_attr(not(feature = "std"), no_std, no_main)]

use ink::prelude::string::String;
use ink::prelude::vec::Vec;
use ink::storage::Mapping;
use scale::{Encode, Decode};

/// Status Enum
#[derive(Encode, Decode, Debug, Clone, PartialEq)]
#[cfg_attr(
    feature = "std",
    derive(scale_info::TypeInfo, ink::storage::traits::StorageLayout)
)]
pub enum Status {
    Active,
    Inactive,
    Graduated,
    Suspended,
}

/// Student Struct
#[derive(Encode, Decode, Debug, Clone)]
#[cfg_attr(
    feature = "std",
    derive(scale_info::TypeInfo, ink::storage::traits::StorageLayout)
)]
pub struct Student {
    id: u32,
    name: String,
    birth_date: String,
    cr: Option<i32>,
    status: Option<Status>,
}

#[ink::contract]
mod student_contract {
    use super::*;

    /// Define student storage
    #[ink(storage)]
    pub struct StudentContract {
        students: Mapping<u32, Student>,
        next_id: u32,
    }

    impl StudentContract {
        /// Student constructor
        /// Initiate mapping
        #[ink(constructor)]
        pub fn new() -> Self {
            Self {
                students: Mapping::default(),
                next_id: 1,
            }
        }

        /// Adds a new student
        #[ink(message)]
        pub fn create_student(
            &mut self,
            name: String,
            birth_date: String,
            cr: Option<i32>,
            status: Option<Status>,
        ) -> u32 {
            let id = self.next_id;
            self.next_id = self
                .next_id
                .checked_add(1)
                .expect("Overflow on next_id increment"); 

            let student = Student {
                id,
                name,
                birth_date,
                cr,
                status,
            };

            self.students.insert(&id, &student);
            id
        }

        /// Gets all students
        #[ink(message)]
        pub fn get_all_students(&self) -> Vec<Student> {
            (1..self.next_id)
                .filter_map(|id| self.students.get(&id))
                .collect()
        }

        /// Gets a specific student by id
        #[ink(message)]
        pub fn get_student(&self, id: u32) -> Option<Student> {
            self.students.get(&id)
        }
        
        /// Updates a student
        #[ink(message)]
        pub fn update_student(
            &mut self,
            id: u32,
            name: Option<String>,
            birth_date: Option<String>,
            cr: Option<i32>,
            status: Option<Option<Status>>,
        ) -> bool {
            if let Some(mut student) = self.students.get(&id) {
                if let Some(new_name) = name {
                    student.name = new_name;
                }
                if let Some(new_birth_date) = birth_date {
                    student.birth_date = new_birth_date;
                }
                if let Some(new_cr) = cr {
                    student.cr = Some(new_cr);
                }
                if let Some(new_status) = status {
                    student.status = new_status;
                }
                self.students.insert(&id, &student);
                true
            } else {
                false
            }
        }
 
        /// Removes a student
        #[ink(message)]
        pub fn delete_student(&mut self, id: u32) -> bool {
            if self.students.get(&id).is_some() {
                self.students.remove(&id);
                true
            } else {
                false
            }
        }
        
    }

    #[cfg(all(test, feature = "e2e-tests"))]
    mod e2e_tests {
        use super::*;

        use ink_e2e::ContractsBackend;

        type E2EResult<T> = std::result::Result<T, Box<dyn std::error::Error>>;

        #[ink_e2e::test]
        async fn default_works(mut client: ink_e2e::Client<C, E>) -> E2EResult<()> {
            let mut constructor = StudentContractRef::new();

            let contract = client
                .instantiate("student_contract", &ink_e2e::alice(), &mut constructor)
                .submit()
                .await
                .expect("instantiate failed");

            let call_builder = contract.call_builder::<StudentContract>();

            let get_call = call_builder.get_all_students();
            let get_result = client.call(&ink_e2e::alice(), &get_call).dry_run().await?;
            assert!(get_result.return_value().is_empty());

            Ok(())
        }

        #[ink_e2e::test]
        async fn crud_works(mut client: ink_e2e::Client<C, E>) -> E2EResult<()> {
            let mut constructor = StudentContractRef::new();

            let contract = client
                .instantiate("student_contract", &ink_e2e::bob(), &mut constructor)
                .submit()
                .await
                .expect("instantiate failed");

            let mut call_builder = contract.call_builder::<StudentContract>();

            // Get all students. Should return []
            let get_call = call_builder.get_all_students();
            let get_result = client.call(&ink_e2e::bob(), &get_call).dry_run().await?;
            assert!(get_result.return_value().is_empty());

            // Creates a new student
            let create_student_call = call_builder.create_student(
                "Test".to_string(),
                "99/99/9999".to_string(),
                Some(8),
                Some(Status::Active),
            );

            let _create_student_result = client
                .call(&ink_e2e::bob(), &create_student_call)
                .submit()
                .await
                .expect("student creation failed");

            // Get student with id 1
            let get_call = call_builder.get_student(1);
            let get_result = client.call(&ink_e2e::bob(), &get_call).dry_run().await?;
            let student = get_result.return_value().unwrap();

            assert_eq!(student.name, "Test");
            assert_eq!(student.birth_date, "99/99/9999");
            assert_eq!(student.cr, Some(8));
            assert_eq!(student.status, Some(Status::Active));

            // Updates a student
            let update_student_call = call_builder.update_student(
                1,
                Some("Test update".to_string()), // Nome como Option<String>
                Some("00/00/0000".to_string()),  // Data de nascimento como Option<String>
                Some(10),            // CR como Option<String> (convertido para String)
                Some(Some(Status::Inactive)),   
            );
            
            let _update_student_result = client
                .call(&ink_e2e::bob(), &update_student_call)
                .submit()
                .await
                .expect("student update failed");

            // Get student with id 1. Should be updated after last call
            let get_call = call_builder.get_student(1);
            let get_result = client.call(&ink_e2e::bob(), &get_call).dry_run().await?;
            let student = get_result.return_value().unwrap();

            assert_eq!(student.name, "Test update");
            assert_eq!(student.birth_date, "00/00/0000");
            assert_eq!(student.cr, Some(10));
            assert_eq!(student.status, Some(Status::Inactive));

            // Removes student with id 1
            let remove_call = call_builder.delete_student(1);
            let remove_result = client.call(&ink_e2e::bob(), &remove_call).submit().await?;
            assert!(matches!(remove_result.return_value(), true));

            // Get all students should be empty again
            let get_call = call_builder.get_all_students();
            let get_result = client.call(&ink_e2e::bob(), &get_call).dry_run().await?;
            assert!(get_result.return_value().is_empty());

            Ok(())
        }
    }
}
