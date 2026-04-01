// SPDX-License-Identifier: AGPL-3.0-or-later
/*
    mze - personal knowledge database
    Copyright (C) 2025  Maksym Medvied

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

const search_url = "/search";

window.addEventListener("load", (e) => {
	const search_box = document.getElementsByClassName("search_box")[0];
	search_box.addEventListener("input", updateSearchResults);
	updateSearchResults(e);
});

// TODO handle concurrent queries properly by cancelling the in-progress search
// and executing a new one
async function updateSearchResults(e) {
	const search_box = document.getElementsByClassName("search_box")[0];

	const search_params = new URLSearchParams({
		q: search_box.value,
	});
	const url = search_url + "?" + search_params.toString();
	console.log("123");
	try {
		const response = await fetch(url);
		if (!response.ok) {
			console.error(`Error: url=${url} response.status=${response.status}`);
		}
		const json = await response.json();
		console.log(json)
		document
			.getElementsByClassName("search_interpretation")[0]
			.innerHTML = json.search_interpretation;
		document
			.getElementsByClassName("search_results_tags")[0]
			.innerHTML = json.search_results.search_results_tags;
		document
			.getElementsByClassName("search_results_attributes")[0]
			.innerHTML = json.search_results.search_results_attributes;
		document
			.getElementsByClassName("search_results_records_and_links")[0]
			.innerHTML = json.search_results.search_results_records_and_links;
	} catch (error) {
		console.error(`Error: error.message=${error.message}`);
	}
}

