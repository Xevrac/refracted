using System;
using Rts.Serialization.StreamExtensions;

namespace GeneratedSerializers.CurrentVersion.Rts_CnC_Messages_Client_TechTreeEntityActivation
{
    public class Serializer
    {
        public static Type SerializedType{ get { return typeof(Rts.CnC.Messages.Client.TechTreeEntityActivation); } }
        
        public static void Serialize(System.IO.Stream s, object obj)
        {
            var value = (Rts.CnC.Messages.Client.TechTreeEntityActivation)obj;
            //  Serialize DependencyType
            s.Write(value.DependencyType);
            //  Serialize EntityType
            s.Write(value.EntityType);
            //  Serialize Unlocked
            s.Write(value.Unlocked);
            //  Serialize EntityDependency
            s.Write(value.EntityDependency);
            //  Serialize Interchangeable
            s.Write(value.Interchangeable);
            //  Serialize InstanceId
            s.Write(value.InstanceId);

        }
        
        public static object Deserialize(System.IO.Stream s)
        {
            var value = System.Runtime.Serialization.FormatterServices.GetUninitializedObject(typeof(Rts.CnC.Messages.Client.TechTreeEntityActivation)) as Rts.CnC.Messages.Client.TechTreeEntityActivation;
            //  Deserialize DependencyType
            s.Read(out value.DependencyType);
            //  Deserialize EntityType
            s.Read(out value.EntityType);
            //  Deserialize Unlocked
            s.Read(out value.Unlocked);
            //  Deserialize EntityDependency
            s.Read(out value.EntityDependency);
            //  Deserialize Interchangeable
            s.Read(out value.Interchangeable);
            //  Deserialize InstanceId
            s.Read(out value.InstanceId);

            return value;
        }
        
    }
}
