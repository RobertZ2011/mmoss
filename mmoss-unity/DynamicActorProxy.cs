using UnityEngine;

public class DynamicActorProxy : MonoBehaviour
{
    public ulong EntityId;

    public void UpdateTransform(MmossFfi.WorldPtr world)
    {
        MmossFfi.Vec3 position;
        MmossFfi.Quat rotation;
        MmossFfi.mmoss_dynamic_actor_proxy_get_tranform(
            world,
            this.EntityId,
            out position,
            out rotation);
        gameObject.transform.position = new Vector3(position.x, position.y, position.z);
        gameObject.transform.rotation = Quaternion.AngleAxis(rotation.w, new Vector3(rotation.x, rotation.y, rotation.z));
    }

    // Start is called once before the first execution of Update after the MonoBehaviour is created
    void Start()
    {
        
    }

    // Update is called once per frame
    void Update()
    {
        
    }
}
