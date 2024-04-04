window.addEventListener("load", (event) => {
  'use strict';

  // document status as CSS class on body
  const doc_status = DOCDUSTRY_LOCALS["status"];
  if (doc_status) {
    document.querySelector('body').classList.add("status-" + doc_status);
  }

  // ids for top headline
  const first_h1 = document.querySelector('h1');
  const doc_id = DOCDUSTRY_LOCALS["did"];
  if (first_h1) {
    first_h1.setAttribute("id", doc_id);
  }

  const main = document.querySelector('section.main');
  console.log("main:", main);
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
    const dummy = document.createElement('div');
    tocContainer.appendChild(dummy);
    var currentLevel = 0;
    let currentList = tocContainer;

    headings.forEach((heading, index) => {
      if (!heading.id) {
        heading.id = "toc-" + index;
      }

      var level = parseInt(heading.tagName.charAt(1));

      // Adjust level based on section nesting
      var sectionCount = 0;
      var parent = heading.parentElement;
      while (parent && parent !== main) {
        if (parent.tagName.toLowerCase() === 'article') {
          sectionCount++;
        }
        parent = parent.parentElement;
      }
      level += sectionCount;


      // Create new list if necessary
      while (currentLevel < level) {
        var newList = document.createElement('ul');
        const last = currentList.lastElementChild;
        if (last) {
          currentList.lastElementChild.appendChild(newList);
        } else {
          const newItem = document.createElement('li');
          currentList.appendChild(newItem);
          newItem.appendChild(newList);
        }
        currentList = newList;
        currentLevel++;
      }

      // Go up levels if necessary
      while (currentLevel > level) {
        currentList = currentList.parentElement.parentElement;
        currentLevel--;
      }

      // Create list item and link
      var listItem = document.createElement('li');
      var link = document.createElement('a');
      link.textContent = heading.textContent;
      link.href = '#' + heading.id;
      listItem.appendChild(link);
      currentList.appendChild(listItem);

    });
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
  console.log("searching via", searchInput, searchResultsContainer);
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
      console.log("render results");
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

  // backlinks
  if (main) {
    const backlinks = [];
    DOCDUSTRY_GLOBALS.docs.forEach(doc => {
      for (const link of doc.links) {
        if (!link.startsWith("did:")) continue;
        if (link.substring(4) === doc_id) {
          backlinks.push(doc);
          break;
        }
      }
      for (const link of doc.includes) {
        if (link === doc_id) {
          backlinks.push(doc);
          break;
        }
      }
    });
    if (backlinks.length > 0) {
      const container = document.createElement('div');
      container.classList.add("backlinks");
      const descriptor = document.createElement('span');
      descriptor.classList.add("descriptor");
      descriptor.innerText = "Backlinks:";
      container.appendChild(descriptor);
      for (const doc of backlinks) {
        container.appendChild(document.createTextNode(" "));
        const b = document.createElement('a');
        b.innerText = doc.title;
        b.href = doc.url;
        container.appendChild(b);
      }
      main.appendChild(container);
    }
  }

});
