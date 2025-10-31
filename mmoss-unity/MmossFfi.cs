using System;
using System.Runtime.InteropServices;

public static class MmossFfi
{
    private const string DllName = "mmoss_ffi";

    // Opaque pointer structs
    public struct MobFactoryBuilderPtr
    {
        public IntPtr Handle;
    }

    public struct MobFactoryPtr
    {
        public IntPtr Handle;
    }

    public struct ComponentFactoryBuilderPtr
    {
        public IntPtr Handle;
    }

    public struct ComponentFactoryPtr
    {
        public IntPtr Handle;
    }

    public struct WorldPtr
    {
        public IntPtr Handle;
    }

    // Delegate types for callbacks
    [UnmanagedFunctionPointer(CallingConvention.Cdecl)]
    public delegate void LogCallback(byte level, string message);

    [UnmanagedFunctionPointer(CallingConvention.Cdecl)]
    public delegate void OnSpawnCallback(ulong entity, uint mobType);

    [UnmanagedFunctionPointer(CallingConvention.Cdecl)]
    public delegate void OnComponentUpdatedCallback(ulong entity, uint id);

    [UnmanagedFunctionPointer(CallingConvention.Cdecl)]
    public delegate void OnComponentAddedCallback(ulong entity, uint spawnId, uint componentType, uint id);

    [DllImport(DllName, CallingConvention = CallingConvention.Cdecl)]
    public static extern void mmoss_init_log(byte level, LogCallback callback);

    [DllImport(DllName, CallingConvention = CallingConvention.Cdecl)]
    public static extern MobFactoryBuilderPtr mmoss_client_factory_builder_new();

    [DllImport(DllName, CallingConvention = CallingConvention.Cdecl)]
    public static extern ComponentFactoryBuilderPtr mmoss_client_component_factory_builder_new();

    [DllImport(DllName, CallingConvention = CallingConvention.Cdecl)]
    public static extern ComponentFactoryPtr mmoss_client_component_factory_builder_build(ComponentFactoryBuilderPtr builder);

    [DllImport(DllName, CallingConvention = CallingConvention.Cdecl)]
    public static extern MobFactoryPtr mmoss_client_factory_builder_build(MobFactoryBuilderPtr builder);

    [DllImport(DllName, CallingConvention = CallingConvention.Cdecl, CharSet = CharSet.Ansi)]
    public static extern WorldPtr mmoss_client_world_new(MobFactoryPtr factory, ComponentFactoryPtr componentFactory, string addr);

    [DllImport(DllName, CallingConvention = CallingConvention.Cdecl)]
    public static extern void mmoss_client_world_destroy(WorldPtr world);

    [DllImport(DllName, CallingConvention = CallingConvention.Cdecl)]
    public static extern void mmoss_client_world_update(
        WorldPtr world,
        OnSpawnCallback onSpawn,
        OnComponentUpdatedCallback onComponentUpdated,
        OnComponentAddedCallback onComponentAdded);

    // FFI-compatible structs
    [StructLayout(LayoutKind.Sequential)]
    public struct Vec3
    {
        public float x;
        public float y;
        public float z;
    }

    [StructLayout(LayoutKind.Sequential)]
    public struct Quat
    {
        public float x;
        public float y;
        public float z;
        public float w;
    }

    [DllImport(DllName, CallingConvention = CallingConvention.Cdecl)]
    public static extern void mmoss_dynamic_actor_proxy_get_tranform(
        WorldPtr world,
        ulong entity,
        out Vec3 out_translation,
        out Quat out_rotation);
}