using System;
using System.Runtime.InteropServices;

public static class MmossExamplesFfi
{
    private const string DllName = "mmoss_examples_lib";

    [DllImport(DllName, CallingConvention = CallingConvention.Cdecl)]
    public static extern void mmoss_examples_lib_register_square_client(MmossFfi.MobFactoryBuilderPtr factory);

    [DllImport(DllName, CallingConvention = CallingConvention.Cdecl)]
    public static extern void mmoss_examples_lib_world_register_components(MmossFfi.WorldPtr world);

    [DllImport(DllName, CallingConvention = CallingConvention.Cdecl)]
    public static extern void moss_examples_lib_register_factory_components(MmossFfi.ComponentFactoryBuilderPtr factory);
}
