window.addEventListener("load", (event) => {
  'use strict';

  // document status as CSS class on body
  const doc_status = DOCDUSTRY_LOCALS["status"];
  if (doc_status) {
    document.querySelector('body').classList.add("status-" + doc_status);
  }

  // ids for top headline
  const first_h1 = document.querySelector('h1');
  const doc_id = DOCDUSTRY_LOCALS["id"];
  if (first_h1) {
    first_h1.setAttribute("id", doc_id);
  }

  const main = document.querySelector('section.main');
  // linking via DID
  if (main) {
    main.querySelectorAll("a[href]").forEach(function(e) {
      var did = e.href;
      if (!did.startsWith("did:")) return;
      did = did.substring(4);
      for (var j = 0, len_j = DOCDUSTRY_GLOBALS.docs.length; j < len_j; j++) {
        const d = DOCDUSTRY_GLOBALS.docs[j];
        if (d.did == did) {
          e.href = d.url;
          e.title = d.title;
          if (e.innerText == "") {
            e.textContent += d.title;
          }
          break;
        }
      }
    });
  }


  // table of contents
  if (main) {
    const headings = main.querySelectorAll('h1, h2, h3, h4, h5, h6');
    const tocContainer = document.createElement('section');
    tocContainer.id = "docdustry-toc";
    tocContainer.style = "float:right";
    const tocList = document.createElement('ul');
    let currentParentList = tocList;
    let previousLevel = 1;
    let id_count = 1;

    headings.forEach((heading) => {
      const listItem = document.createElement('li');
      const anchor = document.createElement('a');
      anchor.textContent = heading.textContent;
      if (!heading.id) {
        heading.id = "toc-" + id_count;
        id_count += 1;
      }
      anchor.href = `#${heading.id}`;
      listItem.appendChild(anchor);

      const level = parseInt(heading.tagName.charAt(1));

      if (level > previousLevel) {
        const sublist = document.createElement('ul');
        currentParentList.lastElementChild.appendChild(sublist);
        currentParentList = sublist;
      } else if (level < previousLevel) {
        let diff = previousLevel - level;
        while (diff > 0) {
          currentParentList = currentParentList.parentElement.parentElement;
          diff--;
        }
      }

      currentParentList.appendChild(listItem);
      previousLevel = level;
    });
    tocContainer.appendChild(tocList);
    main.parentElement.insertBefore(tocContainer, main);
  }

  const header = document.querySelector('header');
  if (header) {
    const search = document.createElement('div');
    search.classList.add("search");
    const field = document.createElement('input');
    field.type = "text";
    field.id = "searchInput";
    field.placeholder = "Search by title or tags...";
    search.appendChild(field);
    const results = document.createElement('div');
    results.id = "searchResults";
    results.classList.add("empty");
    search.appendChild(results);
    header.appendChild(search);
    const nav = document.createElement('nav');
    nav.innerHTML = '<a href="../index.html">Start page</a>';
    header.appendChild(nav);
  }

  const footer = document.querySelector('footer');
  if (footer) {
    footer.innerHTML = "<p>The footer</p>";
  }

  const searchInput = document.getElementById('searchInput');
  const searchResultsContainer = document.getElementById('searchResults');
  // search box
  if (searchInput && searchResultsContainer) {

    function searchDocs(txt) {
      const results = [];
      const lowerCaseQuery = txt.toLowerCase();

      DOCDUSTRY_GLOBALS.docs.forEach(doc => {
        const lowerCaseTitle = doc.title.toLowerCase();
        const tags = doc.tags.map(tag => tag.toLowerCase());

        if (results.length > 20) return;

        if (lowerCaseTitle.includes(lowerCaseQuery) || tags.includes(lowerCaseQuery)) {
          results.push(doc);
        }
      });

      return results;
    }

    function renderResults(results) {
      if (results.length === 0) {
        searchResultsContainer.innerHTML = '<p>No results found.</p>';
        searchResultsContainer.classList.add("empty");
        return;
      } else {
        searchResultsContainer.classList.remove("empty");
      }

      const html = results.map(doc => {
        var url = doc.url;
        if (DOCDUSTRY_LOCALS.did === undefined) {
          url = url.replace('../', '');
        }
        return `<div><a href="${url}">${doc.title}</a></div>`;
      }).join('');

      searchResultsContainer.innerHTML = html;
    }

    searchInput.addEventListener('input', function() {
      const query = this.value.trim();
      if (query.length >= 3) {
        const results = searchDocs(query);
        renderResults(results);
      } else {
        searchResultsContainer.innerHTML = "";
        searchResultsContainer.classList.add("empty");
      }
    });
  }

});
