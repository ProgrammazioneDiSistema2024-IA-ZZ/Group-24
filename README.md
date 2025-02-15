# Backup di Emergenza
Un'applicazione per PC progettata per eseguire backup in caso di schermo inutilizzabile. 
L'utente può attivare il backup tramite un comando grafico tracciato con il mouse, senza la necessità di interagire con una GUI tradizionale.
Supporta anche diverse modalità di backup, inclusi file di tipo specifico e intere cartelle.

## Caratteristiche
L'applicazione offre le seguenti funzionalità:

- **Backup tramite comando grafico**: L'utente può avviare un backup tracciando un rettangolo lungo i bordi dello schermo con il mouse e confermare successivamente l'azione tracciando una riga orizzontale.
    Una volta riconosciuti i due comandi, il programma avvia il processo di backup.

- **Feedback sonoro**: Il programma riproduce suoni nelle seguenti fasi:
  - Suono di attivazione al riconoscimento del comando di backup.
  - Suono di inizio backup.
  - Suono di conferma al completamento del backup.

- **Finestra di conferma con barra di progresso**: Dopo l'attivazione del backup, viene visualizzata una finestra di conferma che segnala che il processo è in corso. All'interno della finestra, è presente una barra di progresso che mostra in tempo reale l'avanzamento del backup, permettendo all'utente di monitorare facilmente il completamento dell'operazione.

- **Selezione del contenuto del backup**: L'utente può scegliere tra diverse opzioni di backup:
  - Backup dell'intero contenuto di una cartella (anche contenente più sottocartelle).
  - Backup di file specifici, selezionando tipi di file come pdf, jpeg, png, ecc.
  - Possibilità di combinare più tipi di file in un singolo backup.

- **Monitoraggio e aggiornamenti in tempo reale**: Il sistema offre un pannello che fornisce:
  - Statistiche di utilizzo della CPU mostrando le statistiche minime, massime e medie dell'uso della CPU, permettendo un'analisi delle prestazioni del sistema in tempo reale.
  - Storico delle operazioni di backup visualizzando i log storici dei backup, con informazioni su durata, dati trasferiti e uso della CPU, consentendo di monitorare l'andamento delle operazioni passate e accedere ai dettagli più significativi.

- **Avvio automatico con il sistema**: Attraverso l'Info Panel, è possibile flaggare una casella per configurare l'avvio automatico dell'applicazione all'accensione del sistema, garantendo che il software sia operativo sin dall'inizio e sempre pronto per eseguire backup.

- **Notifiche di errore e anomalie**: L'applicazione avvisa immediatamente l'utente di eventuali errori o anomalie durante l'esecuzione del backup.

## Compatibilità
L'applicazione è compatibile con i seguenti sistemi operativi:
- Windows 10/11
- Linux (Sviluppato e testato su macchina virtuale con Ubuntu 24.04)

## Download
Per installare l'applicazione, segui questi passaggi:
1. Download: dal repository github andare nella cartella download e scegliere la versione per il sistema operativo desiderato.
2. Estrarre il .zip scaricato

## Avvio
Una volta estratto il .zip il programma è pronto all'uso, basterà lanciare l'applicazione nella cartella estratta.

LINUX: Per il corretto funzionamento dell'applicazione su linux è necessario impostare Ubuntu Xorg all'avvio del sistema opearativo.

![Linux ](images/linux.JPG)

!!! SPOSTARE L'ESEGUBILE DALLA CARTELLA DI INSTALLAZIONE POTREBBE COMPROMETTERE IL CORRETTO FUNZIONAMENTO DEL PROGRAMMA!!!
## Schermate
Ecco alcune schermate dell'applicazione:

La prima schermata (Info Panel) fornisce una panoramica generale dell'applicazione e delle sue funzionalità principali.

![Info Panel ](images/InfoPanel.png)

Nella schermata di configurazione del backup puoi selezionare la cartella di origine (source folder), quella di destinazione (destination folder) e specificare le estensioni dei file da salvare (ad esempio .txt, .jpg), oppure lasciare 'all' per includere tutti i file.
![Backup_Panel](images/backuppanel.png)

Puoi scegliere "Custom" per specificare manualmente le estensioni da salvare (ad esempio .txt, .jpg)
![Configuration](images/customFiles.png)

Appena avviato il backup, viene mostrata una barra di progresso che indica lo stato dell'operazione, consentendo di monitorare in tempo reale l'avanzamento del processo; inoltre, hai la possibilità di interrompere il backup in qualsiasi momento tramite il bottone stop.
![Backup_Progress](images/backupProgress.png)

Al termine del backup, viene visualizzata una schermata di conferma che indica il completamento dell'operazione.
![Backup_Completed](images/backupCompleted.png)

Questa schermata consente di monitorare le performance del sistema e tenere traccia della cronologia dei backup.
![Analytics](images/analyticsPanel.png)






