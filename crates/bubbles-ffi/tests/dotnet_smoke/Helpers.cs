using System.Runtime.InteropServices;
using System.Text;

/// RAII wrapper that pins a UTF-8 encoded string for the duration of a native call.
internal struct Utf8Pin : IDisposable
{
    private GCHandle _handle;
    public nint  Ptr { get; }
    public nuint Len { get; }

    public Utf8Pin(string s)
    {
        var bytes = Encoding.UTF8.GetBytes(s);
        _handle = GCHandle.Alloc(bytes, GCHandleType.Pinned);
        Ptr = _handle.AddrOfPinnedObject();
        Len = (nuint)bytes.Length;
    }

    public void Dispose()
    {
        if (_handle.IsAllocated) _handle.Free();
    }
}

internal static class Helpers
{
    /// Returns the last library error as a string (pointer valid until next bubbles_* call).
    public static string LastError() =>
        Marshal.PtrToStringUTF8(Native.bubbles_last_error()) ?? "(no error)";

    /// Compile a .bub script; prints the error and returns Zero on failure.
    public static nint Compile(string script)
    {
        using var pin = new Utf8Pin(script);
        var rc = Native.bubbles_compile(pin.Ptr, pin.Len, out var program);
        if (rc != Native.Ok || program == nint.Zero)
        {
            Console.Error.WriteLine($"bubbles_compile failed: {LastError()}");
            return nint.Zero;
        }
        return program;
    }

    /// Start a named node; prints the error and returns 1 on failure.
    public static int StartNode(nint runner, string node)
    {
        using var pin = new Utf8Pin(node);
        var rc = Native.bubbles_runner_start(runner, pin.Ptr, pin.Len);
        if (rc != Native.Ok)
            Console.Error.WriteLine($"bubbles_runner_start(\"{node}\") failed: {LastError()}");
        return rc == Native.Ok ? 0 : 1;
    }

    /// Drain all events to completion. Returns false if next_event returned Err.
    public static bool DrainAll(nint runner)
    {
        while (true)
        {
            var ev = Native.bubbles_runner_next_event(runner, out var ptr);
            if (ev == Native.Done) return true;
            if (ev != Native.Ok)  return false;
            if (ptr != nint.Zero) Native.bubbles_string_free(ptr);
        }
    }

    /// Drain events until a Line kind is seen (returns true) or DONE (returns false).
    public static bool DrainUntilLine(nint runner)
    {
        while (true)
        {
            var ev = Native.bubbles_runner_next_event(runner, out var ptr);
            if (ev == Native.Done) return false;
            if (ev != Native.Ok || ptr == nint.Zero) return false;
            var json = Marshal.PtrToStringUTF8(ptr);
            Native.bubbles_string_free(ptr);
            if (json != null && json.Contains("\"kind\":\"Line\"", StringComparison.Ordinal))
                return true;
        }
    }

    /// Drain all events to completion, returning every event JSON string.
    public static List<string> DrainAndCollect(nint runner)
    {
        var events = new List<string>();
        while (true)
        {
            var ev = Native.bubbles_runner_next_event(runner, out var ptr);
            if (ev == Native.Done || ev != Native.Ok) break;
            var json = Marshal.PtrToStringUTF8(ptr);
            Native.bubbles_string_free(ptr);
            if (json != null) events.Add(json);
        }
        return events;
    }

    /// Read a variable as its JSON representation (e.g. {"Number":1.0}), or null on error.
    public static string? ReadVariable(nint runner, string name)
    {
        using var pin = new Utf8Pin(name);
        if (Native.bubbles_runner_variable_get_json(runner, pin.Ptr, pin.Len, out var ptr) != Native.Ok)
            return null;
        var s = Marshal.PtrToStringUTF8(ptr);
        Native.bubbles_string_free(ptr);
        return s;
    }

    /// Write a variable from its JSON representation (e.g. "42", "\"Aria\"", "true").
    public static bool WriteVariable(nint runner, string name, string valueJson)
    {
        using var namePin = new Utf8Pin(name);
        using var valPin  = new Utf8Pin(valueJson);
        return Native.bubbles_runner_variable_set_json(
            runner, namePin.Ptr, namePin.Len, valPin.Ptr, valPin.Len) == Native.Ok;
    }
}
