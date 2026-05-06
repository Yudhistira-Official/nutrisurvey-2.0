Option Explicit

Dim shell, fso
Dim baseDir, batPath, installPath, htaPath
Dim readyPath, step1Path, step2Path, step3Path, step4Path
Dim htaProc, htaPid
Dim ready, i

Set shell = CreateObject("WScript.Shell")
Set fso = CreateObject("Scripting.FileSystemObject")

baseDir = fso.GetParentFolderName(fso.GetParentFolderName(WScript.ScriptFullName))
batPath = baseDir & "\Assets\RUN.bat"
installPath = baseDir & "\Assets\INSTALL.bat"
htaPath = baseDir & "\Assets\loading.hta"
readyPath = baseDir & "\Assets\launch.ready"
step1Path = baseDir & "\Assets\launch.step1"
step2Path = baseDir & "\Assets\launch.step2"
step3Path = baseDir & "\Assets\launch.step3"
step4Path = baseDir & "\Assets\launch.step4"

CleanupMarkers

If Not IsDependencyReady() Then
    MsgBox ".NET 8 belum terdeteksi. Installer akan dijalankan.", 64, "NutriSurvey 2.0"
    shell.Run Chr(34) & installPath & Chr(34), 1, True
    If Not IsDependencyReady() Then
        MsgBox "Dependensi belum siap. Jalankan installer sampai selesai, lalu buka Assets\\NutriSurvey.vbs lagi.", 48, "NutriSurvey 2.0"
        WScript.Quit 1
    End If
End If

WriteLoaderHta

Set htaProc = shell.Exec("mshta.exe " & Chr(34) & htaPath & Chr(34))
htaPid = htaProc.ProcessID

shell.Run Chr(34) & batPath & Chr(34), 0, False

ready = False
For i = 1 To 120
    WScript.Sleep 1000
    If IsUrlReady("http://localhost:8080") Then
        ready = True
        Exit For
    End If
Next

If ready Then
    For i = 1 To 30
        If IsEdgeOpened() Then Exit For
        WScript.Sleep 500
    Next

    On Error Resume Next
    Dim rf
    Set rf = fso.CreateTextFile(readyPath, True)
    rf.WriteLine "ready"
    rf.Close
    On Error GoTo 0
    WScript.Sleep 400
End If

On Error Resume Next
If htaPid > 0 Then shell.Run "taskkill /F /PID " & htaPid & " >nul 2>&1", 0, True
CleanupMarkers
On Error GoTo 0

Sub CleanupMarkers()
    On Error Resume Next
    If fso.FileExists(readyPath) Then fso.DeleteFile readyPath, True
    If fso.FileExists(step1Path) Then fso.DeleteFile step1Path, True
    If fso.FileExists(step2Path) Then fso.DeleteFile step2Path, True
    If fso.FileExists(step3Path) Then fso.DeleteFile step3Path, True
    If fso.FileExists(step4Path) Then fso.DeleteFile step4Path, True
    On Error GoTo 0
End Sub

Sub WriteLoaderHta()
    Dim hta
    Dim html
    Dim markerBaseJs
    Set hta = fso.CreateTextFile(htaPath, True)
    markerBaseJs = Replace(baseDir & "\Assets", "\", "\\")

    html = "<html><head><title>NutriSurvey Loader</title>" & _
        "<hta:application id='app' caption='no' border='none' showintaskbar='no' singleinstance='yes' sysmenu='no' maximizebutton='no' minimizebutton='no' scroll='no' />" & _
        "<style>html,body{width:100%;height:100%;overflow:hidden;}body{font-family:'Plus Jakarta Sans','Poppins','Segoe UI',Arial,sans-serif;margin:0;background:linear-gradient(160deg,#f3fbf6 0%,#ebf7ef 100%);color:#123524;}" & _
        ".wrap{padding:20px 22px;} .title{margin:0 0 6px 0;font-size:24px;font-weight:700;} .sub{margin:0 0 12px 0;color:#2f5f49;font-size:15px;}" & _
        ".top{display:flex;justify-content:space-between;align-items:center;margin-bottom:10px;} .pct{font-size:18px;font-weight:700;color:#0a7a3e;}" & _
        ".track{height:16px;border-radius:999px;background:#dfeee5;overflow:hidden;box-shadow:inset 0 0 0 1px #cde2d6;} .fill{height:100%;width:0%;background:linear-gradient(90deg,#00a84f,#3ac17a);transition:width .25s ease;}" & _
        ".stage{margin-top:8px;font-size:13px;color:#3c6d55;}" & _
        "</style>" & _
        "<script>function setPct(v){document.getElementById('fill').style.width=v+'%';document.getElementById('pct').innerText=v+' %';" & _
        "var st='Menyiapkan...';if(v===25)st='Task 1/4: cek komponen';if(v===50)st='Task 2/4: backend jalan';if(v===75)st='Task 3/4: frontend jalan';if(v===100)st='Task 4/4: membuka browser';document.getElementById('stage').innerText=st;}" & _
        "var BASE='" & markerBaseJs & "';" & _
        "function chk(){try{var fso=new ActiveXObject('Scripting.FileSystemObject');var p=0;if(fso.FileExists(BASE+'\\\\launch.step1'))p=25;if(fso.FileExists(BASE+'\\\\launch.step2'))p=50;if(fso.FileExists(BASE+'\\\\launch.step3'))p=75;if(fso.FileExists(BASE+'\\\\launch.step4'))p=100;" & _
        "if(fso.FileExists(BASE+'\\\\launch.ready')){setPct(100);setTimeout(function(){window.close();},350);return;}setPct(p);}catch(e){document.getElementById('stage').innerText='Menunggu proses...';}}" & _
        "window.onload=function(){try{window.resizeTo(620,210);var x=(screen.availWidth-620)/2;var y=(screen.availHeight-210)/2;window.moveTo(x,y);}catch(e){}setPct(0);setInterval(chk,150);};</script></head>" & _
        "<body><div class='wrap'><div class='top'><h3 class='title'>NutriSurvey 2.0</h3><span id='pct' class='pct'>0 %</span></div><p class='sub'>Membuka aplikasi, mohon tunggu sebentar...</p><div class='track'><div id='fill' class='fill'></div></div><div id='stage' class='stage'>Menyiapkan...</div></div></body></html>"

    hta.WriteLine html
    hta.Close
End Sub

Function IsUrlReady(url)
    On Error Resume Next
    Dim http
    Set http = CreateObject("MSXML2.ServerXMLHTTP.6.0")
    http.setTimeouts 500, 500, 1000, 1000
    http.open "GET", url, False
    http.send ""
    IsUrlReady = (Err.Number = 0 And http.status >= 200 And http.status < 500)
    Err.Clear
    On Error GoTo 0
End Function

Function IsEdgeOpened()
    On Error Resume Next
    Dim svc, ps, p
    IsEdgeOpened = False
    Set svc = GetObject("winmgmts:\\.\root\cimv2")
    Set ps = svc.ExecQuery("Select Name, CommandLine from Win32_Process where Name='msedge.exe'")
    For Each p In ps
        If InStr(1, LCase(p.CommandLine & ""), "localhost:8080", vbTextCompare) > 0 Then
            IsEdgeOpened = True
            Exit For
        End If
    Next
    On Error GoTo 0
End Function

Function IsDependencyReady()
    On Error Resume Next
    Dim code
    code = shell.Run("cmd /c dotnet --version >nul 2>&1", 0, True)
    IsDependencyReady = (Err.Number = 0 And code = 0)
    Err.Clear
    On Error GoTo 0
End Function
