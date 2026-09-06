# KidOS interactive reboot validation

The normal GitHub-hosted Windows workflow can verify installation, services, recovery, permissions, and uninstall, but it cannot reproduce a real child desktop login after Windows Assigned Access takes control of the session.

This validation is designed for a disposable Windows 11 test VM or physical test machine with a self-hosted GitHub Actions runner registered with the label `kidos-interactive-test`.

## Before running the workflow

1. Install the current KidOS Windows installer on the disposable machine.
2. Create or select a standard non-administrator child account.
3. Use the KidOS Parent Dashboard to set the parent PIN and apply Windows lockdown to that child account.
4. Reboot Windows.
5. Sign in interactively to the child account.
6. Confirm KidOS opens in that child session.
7. Leave the child session signed in and switch back to an administrator session without signing the child out.

The self-hosted GitHub runner should run as a Windows service so it remains available while the child session is signed in.

## Run the validation

Open **Actions → KidOS Interactive Windows Session Validation → Run workflow**.

Enter the exact Windows child account name and the KidOS shell executable name. The workflow checks:

- KidOS Guardian survived the reboot and is running.
- KidOS Media Classifier survived the reboot and is running.
- Windows Assigned Access still has a configuration.
- The Assigned Access configuration targets the expected child account.
- Windows recorded a recent interactive logon for that child.
- The KidOS shell process is actually running under the child account.

A passing run uploads `interactive-session-result.json` as the **KidOS-Interactive-Session-Validation** artifact.

## Release rule

Do not call a KidOS build fully validated for production Windows lockdown until both of these pass:

1. **KidOS Clean Windows VM CI**
2. **KidOS Interactive Windows Session Validation**

The second check must be performed on a disposable Windows environment because it intentionally exercises a real locked child session.
