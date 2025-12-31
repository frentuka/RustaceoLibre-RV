#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[ink::contract]
mod rustaceo_libre_rv {

    use ink::env::call::FromAccountId;
    use ink::prelude::vec::Vec;

    use rustaceo_libre::rustaceo_libre::RustaceoLibreRef;
    use rustaceo_libre::structs::producto::CategoriaProducto;

    #[ink(storage)]
    pub struct RustaceoLibreRV {
        rl_address: AccountId,
    }

    impl RustaceoLibreRV {
        #[ink(constructor)]
        pub fn new(rl_address: AccountId) -> Self {
            Self { rl_address }
        }

        pub fn get_rl(&self) -> RustaceoLibreRef {
            RustaceoLibreRef::from_account_id(self.rl_address)
        }

        /// Consultar top 5 compradores con mejor reputación.
        #[ink(message)]
        pub fn top5_compradores(&self) -> Vec<(AccountId, u8)> {
            let rl = self.get_rl();

            let compradores = rl.ver_usuarios_compradores();
            let mut ids_calificaciones: Vec<_> = compradores.iter().filter_map(|&id| {
                if let Some(cal) = rl.ver_calificacion_comprador(id) { Some((id, cal)) }
                else { None }
            }).collect();

            ids_calificaciones.sort_by_key(|&(_, v)| core::cmp::Reverse(v));
            ids_calificaciones.iter().take(5).cloned().collect()
        }

        /// Consultar top 5 vendedores con mejor reputación.
        #[ink(message)]
        pub fn top5_vendedores(&self) -> Vec<(AccountId, u8)> {
            let rl = self.get_rl();

            let vendedores = rl.ver_usuarios_vendedores();
            let mut ids_calificaciones: Vec<_> = vendedores.iter().filter_map(|&id| {
                if let Some(cal) = rl.ver_calificacion_vendedor(id) { Some((id, cal)) }
                else { None }
            }).collect();

            ids_calificaciones.sort_by_key(|&(_, v)| core::cmp::Reverse(v));
            ids_calificaciones.iter().take(5).cloned().collect()
        }

        /// Ver 'top' productos más vendidos.
        #[ink(message)]
        pub fn productos_mas_vendidos(&self, top: u128) -> Vec<(u128, u128)> /* (idProducto, ventas) */ {
            let rl = self.get_rl();

            let productos = rl.ver_id_productos();
            let mut prod_ventas: Vec<_> = productos.iter().filter_map(|&id| {
                if let Some(cal) = rl.ver_ventas_producto(id) { Some((id, cal)) }
                else { None }
            }).collect();

            prod_ventas.sort_by_key(|&(_, v)| core::cmp::Reverse(v));
            prod_ventas.iter().take(top as usize).cloned().collect()
        }

        /// Cantidad de órdenes por usuario.
        #[ink(message)]
        pub fn ordenes_del_usuario(&self, user: AccountId) -> Option<u128> {
            let rl = self.get_rl();
            rl.ver_cantidad_compras(user)
        }

        /// Estadísticas por categoría: total de ventas, calificación promedio.
        /// Devuelve (calificacionPromedio, ventas)
        #[ink(message)]
        pub fn estadisticas_por_categoria(&self, categoria: CategoriaProducto) -> Option<(u128, u128)> {
            let rl = self.get_rl();

            // encontrar calificación promedio
            let mut amt_calif = 0u128;
            let mut total_sum_calif = 0u128;
            for id_venta in rl.ver_id_pedidos() {
                let Some(calificacion) = rl.ver_calificacion_comprador_pedido(id_venta)
                else { continue; };

                let Some(new_sum_calif) = total_sum_calif.checked_add(calificacion as u128)
                else { continue; };

                let Some(new_amt_calif) = amt_calif.checked_add(1)
                else { continue; };

                total_sum_calif = new_sum_calif;
                amt_calif = new_amt_calif;
            }

            let promedio = if amt_calif != 0 {
                total_sum_calif.div_euclid(amt_calif)
            } else {
                0
            };

            // encontrar cantidad de ventas
            let mut total_ventas = 0u128;
            for id_prod in rl.ver_id_productos() {
                let Some(producto) = rl.ver_producto(id_prod)
                else { continue; };

                if producto.categoria != categoria {
                    continue;
                }

                let Some(new_total_ventas) = total_ventas.checked_add(producto.ventas)
                else { continue; };

                total_ventas = new_total_ventas;
            }

            Some((promedio, total_ventas))
        }
    }

    #[cfg(test)]
    mod tests {

    }
}
