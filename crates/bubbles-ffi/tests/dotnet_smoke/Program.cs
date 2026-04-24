using System.Globalization;
using System.Runtime.CompilerServices;
using System.Runtime.InteropServices;
using System.Text;
using System.Text.Json;

internal static unsafe class Native
{
    private const string Dll = "bubbles_ffi";

    public const int Ok = 0;
    public const int Done = 1;
    public const int Err = -1;

    public const int SaliencyFirstAvailable = 0;
    public const int SaliencyBlrv = 1;

    [DllImport(Dll, CallingConvention = CallingConvention.Cdecl)]
    public static extern uint bubbles_abi_version();

    [DllImport(Dll, CallingConvention = CallingConvention.Cdecl)]
    public static extern nint bubbles_copy_utf8(nint ptr, nuint len);

    [DllImport(Dll, CallingConvention = CallingConvention.Cdecl)]
    public static extern int bubbles_compile(
        nint textPtr,
        nuint textLen,
        out nint outProgram);

    [DllImport(Dll, CallingConvention = CallingConvention.Cdecl)]
    public static extern int bubbles_program_node_exists(
        nint program,
        nint nodePtr,
        nuint nodeLen,
        out int outExists);

    [DllImport(Dll, CallingConvention = CallingConvention.Cdecl)]
    public static extern int bubbles_runner_new(nint program, out nint outRunner);

    [DllImport(Dll, CallingConvention = CallingConvention.Cdecl)]
    public static extern int bubbles_runner_new_with_saliency(
        nint program,
        int saliencyKind,
        out nint outRunner);

    [DllImport(Dll, CallingConvention = CallingConvention.Cdecl)]
    public static extern void bubbles_runner_free(nint runner);

    [DllImport(Dll, CallingConvention = CallingConvention.Cdecl)]
    public static extern int bubbles_runner_set_locale_json(nint runner, nint jsonPtr, nuint jsonLen);

    [DllImport(Dll, CallingConvention = CallingConvention.Cdecl)]
    public static extern int bubbles_runner_register_function(
        nint runner,
        nint namePtr,
        nuint nameLen,
        delegate* unmanaged[Cdecl]<nint, nint, nuint, nint*, int> cb,
        nint userdata);

    [DllImport(Dll, CallingConvention = CallingConvention.Cdecl)]
    public static extern int bubbles_runner_start(nint runner, nint nodePtr, nuint nodeLen);

    [DllImport(Dll, CallingConvention = CallingConvention.Cdecl)]
    public static extern int bubbles_runner_next_event(nint runner, out nint outJson);

    [DllImport(Dll, CallingConvention = CallingConvention.Cdecl)]
    public static extern int bubbles_runner_variable_get_json(
        nint runner,
        nint namePtr,
        nuint nameLen,
        out nint outJson);

    [DllImport(Dll, CallingConvention = CallingConvention.Cdecl)]
    public static extern void bubbles_string_free(nint p);
}

internal static unsafe class Program
{
    private const string ScriptBasic = """
title: Start
---
Alice: Hi
===
""";

    private const string ScriptHost = """
title: T
---
<<set $n = add_one(41)>>
NPC: done
===
""";

    private const string ScriptLocale = """
title: L
---
Shopkeep: Hello #line:greet
===
""";

    private static int Main()
    {
        if (Native.bubbles_abi_version() != 1)
        {
            Console.Error.WriteLine("unexpected bubbles_abi_version (expected 1)");
            return 2;
        }

        if (RunBasic() != 0) return 1;
        if (RunHostAndVariables() != 0) return 1;
        if (RunLocale() != 0) return 1;

        Console.WriteLine("dotnet smoke ok");
        return 0;
    }

    private static int RunBasic()
    {
        var program = Compile(ScriptBasic);
        if (program == nint.Zero) return 1;

        var node = "Start"u8.ToArray();
        var nh = GCHandle.Alloc(node, GCHandleType.Pinned);
        int exists;
        try
        {
            var rc = Native.bubbles_program_node_exists(
                program,
                nh.AddrOfPinnedObject(),
                (nuint)node.Length,
                out exists);
            if (rc != Native.Ok || exists != 1)
            {
                Console.Error.WriteLine("bubbles_program_node_exists failed");
                return 1;
            }
        }
        finally
        {
            nh.Free();
        }

        if (Native.bubbles_runner_new(program, out var runner) != Native.Ok || runner == nint.Zero)
        {
            Console.Error.WriteLine("bubbles_runner_new failed");
            return 1;
        }

        if (StartNode(runner, "Start") != 0)
        {
            Native.bubbles_runner_free(runner);
            return 1;
        }

        if (!DrainUntilLine(runner))
        {
            Native.bubbles_runner_free(runner);
            return 1;
        }

        Native.bubbles_runner_free(runner);
        return 0;
    }

    private static unsafe int RunHostAndVariables()
    {
        var program = Compile(ScriptHost);
        if (program == nint.Zero) return 1;

        if (Native.bubbles_runner_new_with_saliency(program, Native.SaliencyBlrv, out var runner) != Native.Ok
            || runner == nint.Zero)
        {
            Console.Error.WriteLine("bubbles_runner_new_with_saliency failed");
            return 1;
        }

        var fname = "add_one"u8.ToArray();
        var fh = GCHandle.Alloc(fname, GCHandleType.Pinned);
        try
        {
            var rr = Native.bubbles_runner_register_function(
                runner,
                fh.AddrOfPinnedObject(),
                (nuint)fname.Length,
                &HostAddOne,
                0);
            if (rr != Native.Ok)
            {
                Console.Error.WriteLine("bubbles_runner_register_function failed");
                Native.bubbles_runner_free(runner);
                return 1;
            }
        }
        finally
        {
            fh.Free();
        }

        if (StartNode(runner, "T") != 0)
        {
            Native.bubbles_runner_free(runner);
            return 1;
        }

        DrainAll(runner);

        var key = "$n"u8.ToArray();
        var kh = GCHandle.Alloc(key, GCHandleType.Pinned);
        try
        {
            var gv = Native.bubbles_runner_variable_get_json(
                runner,
                kh.AddrOfPinnedObject(),
                (nuint)key.Length,
                out var vjson);
            if (gv != Native.Ok)
            {
                Console.Error.WriteLine("variable_get failed");
                Native.bubbles_runner_free(runner);
                return 1;
            }

            var vs = Marshal.PtrToStringUTF8(vjson);
            Native.bubbles_string_free(vjson);
            if (vs != "{\"Number\":42.0}")
            {
                Console.Error.WriteLine($"expected $n JSON, got {vs}");
                Native.bubbles_runner_free(runner);
                return 1;
            }
        }
        finally
        {
            kh.Free();
        }

        Native.bubbles_runner_free(runner);
        return 0;
    }

    private static int RunLocale()
    {
        var program = Compile(ScriptLocale);
        if (program == nint.Zero) return 1;

        if (Native.bubbles_runner_new(program, out var runner) != Native.Ok || runner == nint.Zero)
        {
            Console.Error.WriteLine("runner_new (locale) failed");
            return 1;
        }

        var locJson = "{\"greet\":\"Salut\"}"u8.ToArray();
        var lh = GCHandle.Alloc(locJson, GCHandleType.Pinned);
        try
        {
            var lr = Native.bubbles_runner_set_locale_json(
                runner,
                lh.AddrOfPinnedObject(),
                (nuint)locJson.Length);
            if (lr != Native.Ok)
            {
                Console.Error.WriteLine("set_locale_json failed");
                Native.bubbles_runner_free(runner);
                return 1;
            }
        }
        finally
        {
            lh.Free();
        }

        if (StartNode(runner, "L") != 0)
        {
            Native.bubbles_runner_free(runner);
            return 1;
        }

        var saw = false;
        while (true)
        {
            var ev = Native.bubbles_runner_next_event(runner, out var jsonPtr);
            if (ev == Native.Done) break;
            if (ev != Native.Ok)
            {
                Native.bubbles_runner_free(runner);
                return 1;
            }

            var json = Marshal.PtrToStringUTF8(jsonPtr);
            Native.bubbles_string_free(jsonPtr);
            if (json != null && json.Contains("Salut", StringComparison.Ordinal)) saw = true;
        }

        Native.bubbles_runner_free(runner);
        if (!saw)
        {
            Console.Error.WriteLine("expected localised Salut in events");
            return 1;
        }

        return 0;
    }

    private static nint Compile(string script)
    {
        var utf8 = Encoding.UTF8.GetBytes(script);
        var h = GCHandle.Alloc(utf8, GCHandleType.Pinned);
        try
        {
            var rc = Native.bubbles_compile(
                h.AddrOfPinnedObject(),
                (nuint)utf8.Length,
                out var program);
            if (rc != Native.Ok || program == nint.Zero)
            {
                Console.Error.WriteLine("bubbles_compile failed");
                return nint.Zero;
            }

            return program;
        }
        finally
        {
            h.Free();
        }
    }

    private static int StartNode(nint runner, string nodeName)
    {
        var b = Encoding.UTF8.GetBytes(nodeName);
        var h = GCHandle.Alloc(b, GCHandleType.Pinned);
        try
        {
            var rs = Native.bubbles_runner_start(runner, h.AddrOfPinnedObject(), (nuint)b.Length);
            if (rs != Native.Ok)
            {
                Console.Error.WriteLine("bubbles_runner_start failed");
                return 1;
            }
        }
        finally
        {
            h.Free();
        }

        return 0;
    }

    private static bool DrainUntilLine(nint runner)
    {
        while (true)
        {
            var ev = Native.bubbles_runner_next_event(runner, out var jsonPtr);
            if (ev == Native.Done) break;
            if (ev != Native.Ok || jsonPtr == nint.Zero) return false;
            var json = Marshal.PtrToStringUTF8(jsonPtr);
            Native.bubbles_string_free(jsonPtr);
            if (json != null && json.Contains("\"kind\":\"Line\"", StringComparison.Ordinal)) return true;
        }

        return false;
    }

    private static void DrainAll(nint runner)
    {
        while (true)
        {
            var ev = Native.bubbles_runner_next_event(runner, out var jsonPtr);
            if (ev == Native.Done) break;
            if (ev == Native.Ok && jsonPtr != nint.Zero) Native.bubbles_string_free(jsonPtr);
        }
    }

    [UnmanagedCallersOnly(CallConvs = [typeof(CallConvCdecl)])]
    private static unsafe int HostAddOne(nint userdata, nint argsJsonPtr, nuint argsJsonLen, nint* outResultJson)
    {
        try
        {
            var span = new ReadOnlySpan<byte>((void*)argsJsonPtr, (int)argsJsonLen);
            var jsonText = Encoding.UTF8.GetString(span);
            using var doc = JsonDocument.Parse(jsonText);
            var first = doc.RootElement[0];
            double n = 0;
            if (first.ValueKind == JsonValueKind.Number)
                n = first.GetDouble();
            else if (first.ValueKind == JsonValueKind.Object
                     && first.TryGetProperty("Number", out var numEl))
                n = numEl.GetDouble();

            var s = (n + 1.0).ToString(CultureInfo.InvariantCulture);
            var bytes = Encoding.UTF8.GetBytes(s);
            fixed (byte* bp = bytes)
            {
                var p = Native.bubbles_copy_utf8((nint)bp, (nuint)bytes.Length);
                if (p == nint.Zero) return Native.Err;
                *outResultJson = p;
            }

            return Native.Ok;
        }
        catch
        {
            return Native.Err;
        }
    }
}
