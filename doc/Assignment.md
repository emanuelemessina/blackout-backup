
# Assignment

_**Progetto 2.1: Back-up di emergenza**_

Realizzare un’applicazione in Linguaggio RUST per PC utile per permettere di effettuare un back-up
nel caso in cui lo schermo non sia agibile.
\
Qualora l’utente voglia effettuare un backup su un disco esterno (chiavetta USB),
dovrà utilizzare un comando convenzionale attraverso il mouse.
\
La scelta del tipo di comanda è lasciata libera, ma potrebbe essere una X che comprende l’area dello schermo
oppure la composizione di un rettangolo lungo i bordi dello schermo.
\
Dopo aver inviato il comando di attivazione di backup, l’utente deve dare conferma attraverso un secondo comando tramite mouse
(potrebbe essere corrispondente al tracciamento di un + o di un -).
\
La sorgente del back-up può essere indicata in fase di definizione del tool (ad esempio in un file di configurazione).
\
Si richiede anche che dopo che il comando di attivazione del back-up è stato riconosciuto, venga visualizzata nello
schermo una finestra di conferma.
\
Il programma potrebbe prevedere anche diverse tipologie di backup (ad esempio il contenuto di una cartella oppure solo i file di un determinato tipo).
\
Si richiede che tale applicazione sia installata in fase di bootstrap del PC e sia attiva in background.
\
L’applicazione deve avere il minor consumo di CPU possibile.
\
Per valutare il consumo di CPU si richiede che ogni 2 minuti sia salvato su un file di log il consumo di CPU effettuato dall’applicazione.
\
Infine si vuole che al termine dell’esecuzione di backup, nella medesima chiavetta, sia creato un file
di log che specifica la dimensione complessiva dei file salvati ed il tempo di CPU impiegato per il
completamento delle operazioni di backup.
