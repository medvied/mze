// SPDX-License-Identifier: AGPL-3.0-or-later
/*
    mze - personal knowledge database
    Copyright (C) 2024, 2025  Medvied

    This program is free software: you can redistribute it and/or modify
    it under the terms of the GNU Affero General Public License as published by
    the Free Software Foundation, either version 3 of the License, or
    (at your option) any later version.

    This program is distributed in the hope that it will be useful,
    but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
    GNU Affero General Public License for more details.

    You should have received a copy of the GNU Affero General Public License
    along with this program.  If not, see <https://www.gnu.org/licenses/>.
*/

use clap::Parser;

use mze::{app, container, renderer};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Create container (an existing container is opened otherwise)
    #[arg(long, default_value_t = false)]
    container_create: bool,
    /// Container type
    #[arg(long)]
    container_type: String,
    /// URI for Container::new()
    #[arg(long)]
    container_uri: String,
    /// Renderer type
    #[arg(long)]
    renderer_type: String,
    /// URI for Renderer::new()
    #[arg(long, default_value_t = String::from("127.0.0.1:8080"))]
    renderer_uri: String,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    app::init();

    let args = Args::parse();

    let mut container =
        container::new(&args.container_type, &args.container_uri)?;
    if args.container_create {
        container.create()?;
    }

    let mut renderer =
        renderer::new(&args.renderer_type, &args.renderer_uri, container)?;

    renderer.run()
}
