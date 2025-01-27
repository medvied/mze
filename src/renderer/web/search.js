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

