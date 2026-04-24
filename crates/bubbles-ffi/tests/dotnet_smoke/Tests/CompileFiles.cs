using System.Runtime.InteropServices;
using System.Text;

internal static class CompileFilesTest
{
    private const string ScriptA = """
title: FileA
---
NPC: From file A.
===
""";

    private const string ScriptB = """
title: FileB
---
NPC: From file B.
===
""";

    public static unsafe int Run()
    {
        var pathA = "a.bub"u8.ToArray();
        var textA = Encoding.UTF8.GetBytes(ScriptA);
        var pathB = "b.bub"u8.ToArray();
        var textB = Encoding.UTF8.GetBytes(ScriptB);

        // pin all four byte arrays and build the struct array on the stack
        var hpA = GCHandle.Alloc(pathA, GCHandleType.Pinned);
        var htA = GCHandle.Alloc(textA, GCHandleType.Pinned);
        var hpB = GCHandle.Alloc(pathB, GCHandleType.Pinned);
        var htB = GCHandle.Alloc(textB, GCHandleType.Pinned);

        nint program;
        try
        {
            var files = stackalloc Native.SourceFile[2];
            files[0] = new Native.SourceFile
            {
                PathPtr = hpA.AddrOfPinnedObject(), PathLen = (nuint)pathA.Length,
                TextPtr = htA.AddrOfPinnedObject(), TextLen = (nuint)textA.Length,
            };
            files[1] = new Native.SourceFile
            {
                PathPtr = hpB.AddrOfPinnedObject(), PathLen = (nuint)pathB.Length,
                TextPtr = htB.AddrOfPinnedObject(), TextLen = (nuint)textB.Length,
            };

            if (Native.bubbles_compile_files(files, 2, out program) != Native.Ok || program == nint.Zero)
            {
                Console.Error.WriteLine($"CompileFiles: compile_files failed: {Helpers.LastError()}");
                return 1;
            }
        }
        finally
        {
            hpA.Free(); htA.Free(); hpB.Free(); htB.Free();
        }

        // both nodes from both files must be present in the merged program
        using var pinA = new Utf8Pin("FileA");
        using var pinB = new Utf8Pin("FileB");

        if (Native.bubbles_program_node_exists(program, pinA.Ptr, pinA.Len, out var existsA) != Native.Ok
            || existsA != 1)
        {
            Console.Error.WriteLine("CompileFiles: FileA not found in merged program");
            Native.bubbles_program_free(program);
            return 1;
        }
        if (Native.bubbles_program_node_exists(program, pinB.Ptr, pinB.Len, out var existsB) != Native.Ok
            || existsB != 1)
        {
            Console.Error.WriteLine("CompileFiles: FileB not found in merged program");
            Native.bubbles_program_free(program);
            return 1;
        }

        // runner_new consumes program regardless of success/failure
        if (Native.bubbles_runner_new(program, out var runner) != Native.Ok || runner == nint.Zero)
        {
            Console.Error.WriteLine($"CompileFiles: runner_new failed: {Helpers.LastError()}");
            return 1;
        }

        if (Helpers.StartNode(runner, "FileB") != 0) { Native.bubbles_runner_free(runner); return 1; }

        if (!Helpers.DrainAll(runner))
        {
            Console.Error.WriteLine($"CompileFiles: error draining events: {Helpers.LastError()}");
            Native.bubbles_runner_free(runner);
            return 1;
        }

        Native.bubbles_runner_free(runner);
        return 0;
    }
}
