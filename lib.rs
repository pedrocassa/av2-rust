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

    fn validate_birth_date(birth_date: &str) {
        // Verify if it is in the format dd/mm/yyyy
        if !birth_date.chars().all(|c| c.is_digit(10) || c == '/') || birth_date.len() != 10 {
            panic!("A data de nascimento deve estar no formato dd/mm/yyyy");
        }
    
        // Split string into 3 parts
        let parts: Vec<&str> = birth_date.split('/').collect();
        if parts.len() != 3 {
            panic!("A data de nascimento deve estar no formato dd/mm/yyyy");
        }
    
        // Converts into ints
        let day: u32 = parts[0].parse().expect("O dia deve ser um número válido");
        let month: u32 = parts[1].parse().expect("O mês deve ser um número válido");
        let year: u32 = parts[2].parse().expect("O ano deve ser um número válido");
    
        // Validates each part
        if !(1..=31).contains(&day) {
            panic!("O dia deve estar entre 1 e 31");
        }
        if !(1..=12).contains(&month) {
            panic!("O mês deve estar entre 1 e 12");
        }
        if year < 1900 || year > 2100 {
            panic!("O ano deve estar entre 1900 e 2100");
        }
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

            if name.trim().is_empty() {
                panic!("O nome não pode ser vazio");
            }

            if name.len() > 100 {
                panic!("O nome não pode ter mais de 100 caracteres");
            }

            validate_birth_date(&birth_date);

            if let Some(cr) = cr {
                if !(0..=100).contains(&cr) {
                    panic!("O CR deve estar entre 0 e 100");
                }
            }

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
                    if new_name.trim().is_empty() {
                        panic!("O nome não pode ser vazio");
                    }
                    
                    if new_name.len() > 100 {
                        panic!("O nome não pode ter mais de 100 caracteres");
                    }

                    student.name = new_name;
                }
                if let Some(new_birth_date) = birth_date {
                    validate_birth_date(&new_birth_date);

                    student.birth_date = new_birth_date;
                }
                if let Some(new_cr) = cr {
                    if !(0..=100).contains(&new_cr) {
                        panic!("O CR deve estar entre 0 e 100");
                    }

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
    
    #[cfg(test)]
    mod tests {
        use super::*;

        #[ink::test]
        fn create_student_works() {
            let mut contract = StudentContract::new();
            
            let all_students = contract.get_all_students();
            assert!(all_students.is_empty());

            let student_id = contract.create_student(
                "Test".to_string(),
                "01/01/2000".to_string(),
                Some(90),
                Some(Status::Active),
            );

            let student = contract.get_student(student_id);
            assert!(student.is_some());

            let student = student.unwrap();

            assert_eq!(student.name, "Test");
            assert_eq!(student.birth_date, "01/01/2000");
            assert_eq!(student.cr, Some(90));
            assert_eq!(student.status, Some(Status::Active));
        }

        #[ink::test]
        fn update_student_works() {
            let mut contract = StudentContract::new();

            let student_id = contract.create_student(
                "Test".to_string(),
                "02/02/2000".to_string(),
                Some(85),
                Some(Status::Inactive),
            );

            let updated = contract.update_student(
                student_id,
                Some("Test Update".to_string()),
                Some("02/02/2000".to_string()),
                Some(95),
                Some(Some(Status::Active)),
            );

            assert!(updated);

            let student = contract.get_student(student_id).unwrap();

            assert_eq!(student.name, "Test Update");
            assert_eq!(student.birth_date, "02/02/2000");
            assert_eq!(student.cr, Some(95));
            assert_eq!(student.status, Some(Status::Active));
        }

        #[ink::test]
        fn delete_student_works() {
            let mut contract = StudentContract::new();

            let student_id = contract.create_student(
                "Test Delete".to_string(),
                "03/03/2000".to_string(),
                None,
                Some(Status::Graduated),
            );

            let all_students = contract.get_all_students();
            assert_eq!(all_students.len(), 1);

            let deleted = contract.delete_student(student_id);
            assert!(deleted);

            let student = contract.get_student(student_id);
            assert!(student.is_none());
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
