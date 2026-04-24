internal static class Program
{
    private static int Main()
    {
        if (Native.bubbles_abi_version() != 1)
        {
            Console.Error.WriteLine("unexpected bubbles_abi_version (expected 1)");
            return 2;
        }

        if (BasicTest.Run()        != 0) return 1;
        if (CompileFilesTest.Run() != 0) return 1;
        if (OptionsTest.Run()      != 0) return 1;
        if (VariableSetTest.Run()  != 0) return 1;
        if (HostFunctionsTest.Run() != 0) return 1;
        if (LocaleTest.Run()       != 0) return 1;
        if (SaveLoadTest.Run()     != 0) return 1;
        if (MarkupTest.Run()       != 0) return 1;

        Console.WriteLine("dotnet smoke ok");
        return 0;
    }
}
