window.addEventListener("load", (event) => {
     'use strict';

     console.log("GLOBALS: ", DOCDUSTRY_GLOBALS);
     console.log("LOCALS: ", DOCDUSTRY_LOCALS);

     // document status as CSS class on body
     const doc_status = DOCDUSTRY_LOCALS["status"];
     if (doc_status) {
          document.querySelector('body').classList.add("status-"+doc_status);
     }

     // ids for top headline
     const first_h1 = document.querySelector('h1');
     const doc_id = DOCDUSTRY_LOCALS["id"];
     if (first_h1) {
          first_h1.setAttribute("id", doc_id);
     }

     // linking via DID
     const main = document.querySelector('section.main');
     main.querySelectorAll("a[href]").forEach(function(e) {
          var did = e.href;
          if (! did.startsWith("did:")) return;
          did = did.substring(4);
          console.log("Found did link:", e);
          for (var j=0, len_j=DOCDUSTRY_GLOBALS.docs.length; j<len_j; j++) {
               const d = DOCDUSTRY_GLOBALS.docs[j];
               console.log("compare", d.did, "vs", did);
               if (d.did == did) {
                    e.href = d.url;
                    e.title = d.title;
                    console.log("Replaced for", d.url);
                    break;
               }
          }
     });

     // table of contents
     {
         const mainSection = document.querySelector('section.main');
         const headings = mainSection.querySelectorAll('h1, h2, h3, h4, h5, h6');
         const tocContainer = document.createElement('section');
         tocContainer.id = "docdustry-toc";
         tocContainer.style = "float:right";
         const tocList = document.createElement('ul');
         let currentParentList = tocList;
         let previousLevel = 1;

         headings.forEach((heading) => {
             const listItem = document.createElement('li');
             const anchor = document.createElement('a');
             anchor.textContent = heading.textContent;
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
         mainSection.parentElement.insertBefore(tocContainer, mainSection);
     }

     const header = document.querySelector('header');
     header.innerHTML = "<p>The header</p>";

     const footer = document.querySelector('footer');
     footer.innerHTML = "<p>The footer</p>";
});
