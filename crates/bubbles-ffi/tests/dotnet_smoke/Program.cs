using System.Runtime.InteropServices;
using System.Text;

internal static class Native
{
    private const string Dll = "bubbles_ffi";

    public const int Ok = 0;
    public const int Done = 1;
    public const int Err = -1;

    [DllImport(Dll, CallingConvention = CallingConvention.Cdecl)]
    public static extern uint bubbles_abi_version();

    [DllImport(Dll, CallingConvention = CallingConvention.Cdecl)]
    public static extern int bubbles_compile(
        IntPtr textPtr,
        nuint textLen,
        out IntPtr outProgram);

    [DllImport(Dll, CallingConvention = CallingConvention.Cdecl)]
    public static extern int bubbles_runner_new(IntPtr program, out IntPtr outRunner);

    [DllImport(Dll, CallingConvention = CallingConvention.Cdecl)]
    public static extern void bubbles_runner_free(IntPtr runner);

    [DllImport(Dll, CallingConvention = CallingConvention.Cdecl)]
    public static extern int bubbles_runner_start(IntPtr runner, IntPtr nodePtr, nuint nodeLen);

    [DllImport(Dll, CallingConvention = CallingConvention.Cdecl)]
    public static extern int bubbles_runner_next_event(IntPtr runner, out IntPtr outJson);

    [DllImport(Dll, CallingConvention = CallingConvention.Cdecl)]
    public static extern void bubbles_string_free(IntPtr p);
}

internal static class Program
{
    private const string Script = """
title: Start
---
Alice: Hi
===
""";

    private static int Main()
    {
        var ver = Native.bubbles_abi_version();
        if (ver != 1)
        {
            Console.Error.WriteLine($"unexpected bubbles_abi_version: {ver}");
            return 2;
        }

        var scriptUtf8 = Encoding.UTF8.GetBytes(Script);
        var scriptHandle = GCHandle.Alloc(scriptUtf8, GCHandleType.Pinned);
        IntPtr program;
        try
        {
            var textPtr = scriptHandle.AddrOfPinnedObject();
            var rc = Native.bubbles_compile(textPtr, (nuint)scriptUtf8.Length, out program);
            if (rc != Native.Ok || program == IntPtr.Zero)
            {
                Console.Error.WriteLine($"bubbles_compile failed: {rc}");
                return 1;
            }
        }
        finally
        {
            scriptHandle.Free();
        }

        var rn = Native.bubbles_runner_new(program, out var runner);
        if (rn != Native.Ok || runner == IntPtr.Zero)
        {
            Console.Error.WriteLine($"bubbles_runner_new failed: {rn}");
            return 1;
        }

        var startUtf8 = Encoding.UTF8.GetBytes("Start");
        var startHandle = GCHandle.Alloc(startUtf8, GCHandleType.Pinned);
        try
        {
            var nodePtr = startHandle.AddrOfPinnedObject();
            var rs = Native.bubbles_runner_start(runner, nodePtr, (nuint)startUtf8.Length);
            if (rs != Native.Ok)
            {
                Console.Error.WriteLine($"bubbles_runner_start failed: {rs}");
                Native.bubbles_runner_free(runner);
                return 1;
            }
        }
        finally
        {
            startHandle.Free();
        }

        var sawLine = false;
        while (true)
        {
            var ev = Native.bubbles_runner_next_event(runner, out var jsonPtr);
            if (ev == Native.Done)
            {
                break;
            }

            if (ev != Native.Ok || jsonPtr == IntPtr.Zero)
            {
                Console.Error.WriteLine($"bubbles_runner_next_event failed: {ev}");
                Native.bubbles_runner_free(runner);
                return 1;
            }

            var json = Marshal.PtrToStringUTF8(jsonPtr);
            Native.bubbles_string_free(jsonPtr);
            if (json != null && json.Contains("\"kind\":\"Line\"", StringComparison.Ordinal))
            {
                sawLine = true;
            }
        }

        Native.bubbles_runner_free(runner);

        if (!sawLine)
        {
            Console.Error.WriteLine("expected a Line event in the stream");
            return 1;
        }

        Console.WriteLine("dotnet smoke ok");
        return 0;
    }
}
