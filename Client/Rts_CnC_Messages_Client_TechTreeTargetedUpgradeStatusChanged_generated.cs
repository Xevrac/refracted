using System;
using Rts.Serialization.StreamExtensions;

namespace GeneratedSerializers.CurrentVersion.Rts_CnC_Messages_Client_TechTreeTargetedUpgradeStatusChanged
{
    public class Serializer
    {
        public static Type SerializedType{ get { return typeof(Rts.CnC.Messages.Client.TechTreeTargetedUpgradeStatusChanged); } }
        
        public static void Serialize(System.IO.Stream s, object obj)
        {
            var value = (Rts.CnC.Messages.Client.TechTreeTargetedUpgradeStatusChanged)obj;
            //  Serialize PlayerId
            s.Write(value.PlayerId);
            //  Serialize UpgradeType
            s.Write(value.UpgradeType);
            //  Serialize Unlocked
            s.Write(value.Unlocked);
            //  Serialize TooltipStringId
            s.Write(value.TooltipStringId);
            //  Serialize LocalUnlock
            s.Write(value.LocalUnlock);
            //  Serialize InstanceId
            s.Write(value.InstanceId);

        }
        
        public static object Deserialize(System.IO.Stream s)
        {
            var value = System.Runtime.Serialization.FormatterServices.GetUninitializedObject(typeof(Rts.CnC.Messages.Client.TechTreeTargetedUpgradeStatusChanged)) as Rts.CnC.Messages.Client.TechTreeTargetedUpgradeStatusChanged;
            //  Deserialize PlayerId
            s.Read(out value.PlayerId);
            //  Deserialize UpgradeType
            s.Read(out value.UpgradeType);
            //  Deserialize Unlocked
            s.Read(out value.Unlocked);
            //  Deserialize TooltipStringId
            s.Read(out value.TooltipStringId);
            //  Deserialize LocalUnlock
            s.Read(out value.LocalUnlock);
            //  Deserialize InstanceId
            s.Read(out value.InstanceId);

            return value;
        }
        
    }
}
