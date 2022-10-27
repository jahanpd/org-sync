# Org-Sync

A needlessly complex solution for syncing org files across devices in a local area network.

## One-sentence explanation

A local server that detects file changes and communicates to connected devices.

## Details

Using a gossip-sub swarm with a distributed hash table, messages are passed between peers with each file change to alert peers of change and current state of org files recorded in the DHT.

## An important gotcha

There is a tail sitation where forks can occur, whereby edits are made on more than one device while both are disconnected from the swarm.
The solution is to include a warning message on file save in emacs if no peers are recorded at time of save.
There is the option to force push the state of a device to the DHT.
Timestamps are recorded so it is not possible for an older version of a file to automatically overwrite a more current file unless there is a problem with the central timekeeping service.


``` emacs-lisp
(defun my-after-save-actions ()
  "Used in `after-save-hook'."
  (when (memq this-command '(save-buffer save-some-buffers evil-write))
        (message (shell-command-to-string "echo hello"))
    ))


(add-hook 'after-save-hook 'my-after-save-actions)
```
