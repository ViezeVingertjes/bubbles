using System.Runtime.InteropServices;

internal static unsafe class Native
{
    private const string Dll = "bubbles_ffi";

    public const int Ok   =  0;
    public const int Done =  1;
    public const int Err  = -1;

    public const int SaliencyFirstAvailable  = 0;
    public const int SaliencyBlrv            = 1;
    public const int SaliencyRandomAvailable = 2;

    [StructLayout(LayoutKind.Sequential)]
    public struct SourceFile
    {
        public nint  PathPtr;
        public nuint PathLen;
        public nint  TextPtr;
        public nuint TextLen;
    }

    // ── Version / strings ────────────────────────────────────────────────────

    [DllImport(Dll, CallingConvention = CallingConvention.Cdecl)]
    public static extern uint bubbles_abi_version();

    /// Returns a pointer valid until the next bubbles_* call on this thread; do NOT free.
    [DllImport(Dll, CallingConvention = CallingConvention.Cdecl)]
    public static extern nint bubbles_last_error();

    [DllImport(Dll, CallingConvention = CallingConvention.Cdecl)]
    public static extern void bubbles_string_free(nint p);

    [DllImport(Dll, CallingConvention = CallingConvention.Cdecl)]
    public static extern nint bubbles_copy_utf8(nint ptr, nuint len);

    // ── Compile ───────────────────────────────────────────────────────────────

    [DllImport(Dll, CallingConvention = CallingConvention.Cdecl)]
    public static extern int bubbles_compile(nint textPtr, nuint textLen, out nint outProgram);

    [DllImport(Dll, CallingConvention = CallingConvention.Cdecl)]
    public static extern int bubbles_compile_files(SourceFile* files, nuint fileCount, out nint outProgram);

    /// Free a program that was NOT consumed by bubbles_runner_new.
    [DllImport(Dll, CallingConvention = CallingConvention.Cdecl)]
    public static extern void bubbles_program_free(nint program);

    // ── Program inspection ───────────────────────────────────────────────────

    [DllImport(Dll, CallingConvention = CallingConvention.Cdecl)]
    public static extern int bubbles_program_node_exists(
        nint program, nint nodePtr, nuint nodeLen, out int outExists);

    // ── Runner lifecycle ─────────────────────────────────────────────────────

    /// Consumes the program handle; do not call bubbles_program_free afterward.
    [DllImport(Dll, CallingConvention = CallingConvention.Cdecl)]
    public static extern int bubbles_runner_new(nint program, out nint outRunner);

    /// Consumes the program handle; do not call bubbles_program_free afterward.
    [DllImport(Dll, CallingConvention = CallingConvention.Cdecl)]
    public static extern int bubbles_runner_new_with_saliency(
        nint program, int saliencyKind, out nint outRunner);

    [DllImport(Dll, CallingConvention = CallingConvention.Cdecl)]
    public static extern void bubbles_runner_free(nint runner);

    [DllImport(Dll, CallingConvention = CallingConvention.Cdecl)]
    public static extern int bubbles_runner_set_saliency(nint runner, int saliencyKind);

    [DllImport(Dll, CallingConvention = CallingConvention.Cdecl)]
    public static extern int bubbles_runner_set_locale_json(
        nint runner, nint jsonPtr, nuint jsonLen);

    [DllImport(Dll, CallingConvention = CallingConvention.Cdecl)]
    public static extern int bubbles_runner_register_function(
        nint runner, nint namePtr, nuint nameLen,
        delegate* unmanaged[Cdecl]<nint, nint, nuint, nint*, int> cb,
        nint userdata);

    // ── Dialogue loop ─────────────────────────────────────────────────────────

    [DllImport(Dll, CallingConvention = CallingConvention.Cdecl)]
    public static extern int bubbles_runner_start(nint runner, nint nodePtr, nuint nodeLen);

    [DllImport(Dll, CallingConvention = CallingConvention.Cdecl)]
    public static extern int bubbles_runner_next_event(nint runner, out nint outJson);

    [DllImport(Dll, CallingConvention = CallingConvention.Cdecl)]
    public static extern int bubbles_runner_select_option(nint runner, nuint index);

    // ── Variables ────────────────────────────────────────────────────────────

    [DllImport(Dll, CallingConvention = CallingConvention.Cdecl)]
    public static extern int bubbles_runner_variable_get_json(
        nint runner, nint namePtr, nuint nameLen, out nint outJson);

    [DllImport(Dll, CallingConvention = CallingConvention.Cdecl)]
    public static extern int bubbles_runner_variable_set_json(
        nint runner, nint namePtr, nuint nameLen, nint valuePtr, nuint valueLen);

    // ── Save / load ───────────────────────────────────────────────────────────

    [DllImport(Dll, CallingConvention = CallingConvention.Cdecl)]
    public static extern int bubbles_runner_snapshot_session_json(nint runner, out nint outJson);

    [DllImport(Dll, CallingConvention = CallingConvention.Cdecl)]
    public static extern int bubbles_runner_snapshot_storage_json(nint runner, out nint outJson);

    /// Restore storage first, then session.
    [DllImport(Dll, CallingConvention = CallingConvention.Cdecl)]
    public static extern int bubbles_runner_restore_storage_json(
        nint runner, nint jsonPtr, nuint jsonLen);

    [DllImport(Dll, CallingConvention = CallingConvention.Cdecl)]
    public static extern int bubbles_runner_restore_session_json(
        nint runner, nint jsonPtr, nuint jsonLen);
}
