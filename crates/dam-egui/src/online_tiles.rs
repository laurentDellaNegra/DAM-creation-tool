pub struct CartoDarkMatter;

impl walkers::sources::TileSource for CartoDarkMatter {
    fn tile_url(&self, tile_id: walkers::TileId) -> String {
        let subdomains = ["a", "b", "c", "d"];
        let subdomain = subdomains[(tile_id.x as usize + tile_id.y as usize) % subdomains.len()];
        format!(
            "https://{subdomain}.basemaps.cartocdn.com/dark_all/{}/{}/{}.png",
            tile_id.zoom, tile_id.x, tile_id.y
        )
    }

    fn attribution(&self) -> walkers::sources::Attribution {
        walkers::sources::Attribution {
            text: "© CARTO © OpenStreetMap contributors",
            url: "https://carto.com/basemaps/",
            logo_light: None,
            logo_dark: None,
        }
    }
}
