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

     const header = document.querySelector('header');
     header.innerHTML = "<p>The header</p>";

     const footer = document.querySelector('footer');
     footer.innerHTML = "<p>The footer</p>";
});
