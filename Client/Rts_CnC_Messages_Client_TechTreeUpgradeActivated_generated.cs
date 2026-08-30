using System;
using Rts.Serialization.StreamExtensions;

namespace GeneratedSerializers.CurrentVersion.Rts_CnC_Messages_Client_TechTreeUpgradeActivated
{
    public class Serializer
    {
        public static Type SerializedType{ get { return typeof(Rts.CnC.Messages.Client.TechTreeUpgradeActivated); } }
        
        public static void Serialize(System.IO.Stream s, object obj)
        {
            var value = (Rts.CnC.Messages.Client.TechTreeUpgradeActivated)obj;
            //  Serialize DependencyType
            s.Write(value.DependencyType);
            //  Serialize PlayerId
            s.Write(value.PlayerId);
            //  Serialize UpgradeType
            s.Write(value.UpgradeType);
            //  Serialize IsActivated
            s.Write(value.IsActivated);
            //  Serialize Dependency
            s.Write(value.Dependency);
            //  Serialize InstanceId
            s.Write(value.InstanceId);
            //  Serialize Interchangeable
            s.Write(value.Interchangeable);

        }
        
        public static object Deserialize(System.IO.Stream s)
        {
            var value = System.Runtime.Serialization.FormatterServices.GetUninitializedObject(typeof(Rts.CnC.Messages.Client.TechTreeUpgradeActivated)) as Rts.CnC.Messages.Client.TechTreeUpgradeActivated;
            //  Deserialize DependencyType
            s.Read(out value.DependencyType);
            //  Deserialize PlayerId
            s.Read(out value.PlayerId);
            //  Deserialize UpgradeType
            s.Read(out value.UpgradeType);
            //  Deserialize IsActivated
            s.Read(out value.IsActivated);
            //  Deserialize Dependency
            s.Read(out value.Dependency);
            //  Deserialize InstanceId
            s.Read(out value.InstanceId);
            //  Deserialize Interchangeable
            s.Read(out value.Interchangeable);

            return value;
        }
        
    }
}
