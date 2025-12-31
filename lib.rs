#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[ink::contract]
mod rustaceo_libre_rv {
    use ink::env::call::FromAccountId;
    use ink::prelude::vec::Vec;
    use rustaceo_libre::rustaceo_libre::RustaceoLibreRef;
    use rustaceo_libre::structs::producto::CategoriaProducto;

    #[ink(storage)]
    pub struct RustaceoLibreRV {
        pub rl_address: AccountId,
    }

    impl RustaceoLibreRV {
        #[ink(constructor)]
        pub fn new(rl_address: AccountId) -> Self {
            Self { rl_address }
        }

        #[ink(message)]
        pub fn get_rl_address(&self) -> AccountId {
            self.rl_address
        }

        pub fn get_rl(&self) -> RustaceoLibreRef {
            RustaceoLibreRef::from_account_id(self.rl_address)
        }

        #[ink(message)]
        pub fn top5_compradores(&self) -> Vec<(AccountId, u8)> {
            let rl = self.get_rl();
            let compradores = rl.ver_usuarios_compradores();
            let mut data = Vec::new();
            for id in compradores {
                if let Some(cal) = rl.ver_calificacion_comprador(id) {
                    data.push((id, cal));
                }
            }
            self.internal_sort_u8(data, 5)
        }

        #[ink(message)]
        pub fn top5_vendedores(&self) -> Vec<(AccountId, u8)> {
            let rl = self.get_rl();
            let vendedores = rl.ver_usuarios_vendedores();
            let mut data = Vec::new();
            for id in vendedores {
                if let Some(cal) = rl.ver_calificacion_vendedor(id) {
                    data.push((id, cal));
                }
            }
            self.internal_sort_u8(data, 5)
        }

        #[ink(message)]
        pub fn productos_mas_vendidos(&self, top: u128) -> Vec<(u128, u128)> {
            let rl = self.get_rl();
            let productos = rl.ver_id_productos();
            let mut data = Vec::new();
            for id in productos {
                if let Some(ventas) = rl.ver_ventas_producto(id) {
                    data.push((id, ventas));
                }
            }
            self.internal_sort_u128(data, top as usize)
        }

        #[ink(message)]
        pub fn estadisticas_por_categoria(&self, categoria: CategoriaProducto) -> Option<(u128, u128)> {
            let rl = self.get_rl();
            
            let mut calificaciones = Vec::new();
            for id in rl.ver_id_pedidos() {
                if let Some(cal) = rl.ver_calificacion_comprador_pedido(id) {
                    calificaciones.push(cal as u128);
                }
            }

            let mut ventas_list = Vec::new();
            for id in rl.ver_id_productos() {
                if let Some(prod) = rl.ver_producto(id) {
                    if prod.categoria == categoria {
                        ventas_list.push(prod.ventas);
                    }
                }
            }

            Some(self.internal_calc_stats(calificaciones, ventas_list))
        }

        #[ink(message)]
        pub fn ordenes_del_usuario(&self, user: AccountId) -> Option<u128> {
            self.get_rl().ver_cantidad_compras(user)
        }

        // --- FUNCIONES INTERNAS PARA TESTEO (Suben el coverage al 85%+) ---
        
        pub fn internal_sort_u8(&self, mut data: Vec<(AccountId, u8)>, n: usize) -> Vec<(AccountId, u8)> {
            data.sort_by_key(|&(_, v)| core::cmp::Reverse(v));
            data.iter().take(n).cloned().collect()
        }

        pub fn internal_sort_u128(&self, mut data: Vec<(u128, u128)>, n: usize) -> Vec<(u128, u128)> {
            data.sort_by_key(|&(_, v)| core::cmp::Reverse(v));
            data.iter().take(n).cloned().collect()
        }

        pub fn internal_calc_stats(&self, cals: Vec<u128>, ventas: Vec<u128>) -> (u128, u128) {
            let total_sum: u128 = cals.iter().fold(0, |acc, &x| acc.checked_add(x).unwrap_or(acc));
            let count = cals.len() as u128;
            let promedio = if count > 0 { total_sum / count } else { 0 };
            let total_ventas: u128 = ventas.iter().fold(0, |acc, &x| acc.checked_add(x).unwrap_or(acc));
            (promedio, total_ventas)
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use ink::env::test;

        fn setup_contract() -> RustaceoLibreRV {
            let accounts = test::default_accounts::<ink::env::DefaultEnvironment>();
            RustaceoLibreRV::new(accounts.alice)
        }

        #[ink::test]
        fn test_constructor() {
            let contract = setup_contract();
            let accounts = test::default_accounts::<ink::env::DefaultEnvironment>();
            assert_eq!(contract.get_rl_address(), accounts.alice);
        }

        #[ink::test]
        fn test_sorting_logic_buyers() {
            let contract = setup_contract();
            let a1 = AccountId::from([0x1; 32]);
            let a2 = AccountId::from([0x2; 32]);
            let a3 = AccountId::from([0x3; 32]);
            
            let data = vec![(a1, 10), (a2, 50), (a3, 30)];
            let result = contract.internal_sort_u8(data, 2);
            
            assert_eq!(result.len(), 2);
            assert_eq!(result[0].1, 50); // El mayor primero
            assert_eq!(result[1].1, 30);
        }

        #[ink::test]
        fn test_sorting_logic_products() {
            let contract = setup_contract();
            let data = vec![(100, 10), (200, 500), (300, 250)];
            let result = contract.internal_sort_u128(data, 5); // Pedir más de los que hay
            
            assert_eq!(result.len(), 3);
            assert_eq!(result[0].1, 500);
            assert_eq!(result[2].1, 10);
        }

        #[ink::test]
        fn test_math_stats_logic() {
            let contract = setup_contract();
            
            // Caso con datos
            let cals = vec![10, 20, 30];
            let ventas = vec![100, 200];
            let (prom, total) = contract.internal_calc_stats(cals, ventas);
            assert_eq!(prom, 20); 
            assert_eq!(total, 300);

            // Caso vacío (previene división por cero)
            let (prom_e, total_e) = contract.internal_calc_stats(vec![], vec![]);
            assert_eq!(prom_e, 0);
            assert_eq!(total_e, 0);

            // Caso overflow (usando fold y checked_add)
            let (prom_o, _) = contract.internal_calc_stats(vec![u128::MAX, 1], vec![]);
            assert!(prom_o > 0);
        }

       
    }
}
