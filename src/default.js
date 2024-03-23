window.addEventListener("load", (event) => {
     'use strict';

     console.log("GLOBALS: ", DOCDUSTRY_GLOBALS);
     console.log("LOCALS: ", DOCDUSTRY_LOCALS);

     // document status as CSS class on body
     const doc_status = DOCDUSTRY_LOCALS["status"];
     if (doc_status) {
          console.log("doucment status is ", doc_status);
          document.querySelector('body').classList.add("status-"+doc_status);
     }

     // ids for top headline
     const first_h1 = document.querySelector('h1');
     const doc_id = DOCDUSTRY_LOCALS["id"];
     if (first_h1) {
          first_h1.setAttribute("id", doc_id);
     }

     const header = document.querySelector('header');
     header.innerHTML = "<p>The header</p>";

     const footer = document.querySelector('footer');
     footer.innerHTML = "<p>The footer</p>";
});
